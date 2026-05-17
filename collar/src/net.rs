use colored::Colorize;
use current_platform::CURRENT_PLATFORM;
use otr_config::Version;

use otr_core::{Error, Result, SystemError};
use serde::{Deserialize, Serialize};

use crate::{catch, error::CollarError};


const RELEASES_URL: &'static str = "github.com/adrian-randa/otr/releases";
const API_RELEASES_URL: &'static str = "api.github.com/repos/adrian-randa/otr/releases";

#[derive(Debug, Serialize, Deserialize)]
struct GithubApiReleasesResponse(Vec<GithubRelease>);

#[derive(Debug, Serialize, Deserialize)]
struct GithubRelease {
    tag_name: GithubReleaseVersion,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(try_from = "&str")]
struct GithubReleaseVersion(Version);

impl TryFrom<&str> for GithubReleaseVersion {
    type Error = Box<dyn Error>;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        if value[0..=0] != *"v" {
            return Err(SystemError::new("Invalid version number!".into()).boxed());
        }
        Ok(Self(Version::try_from(&value[1..])?))
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

fn get_user_agent() -> String {
    let collar_version = env!("CARGO_PKG_VERSION");
    let platform = CURRENT_PLATFORM;

    format!("Collar/{collar_version} ({platform})")
}

fn fetch_latest_release_version() -> Result<Version> {
    let url = "https://".to_string() + API_RELEASES_URL;
    let user_agent = get_user_agent();

    let client = reqwest::blocking::Client::new();

    let req = client.get(url).header("User-Agent", user_agent);

    let res = catch(req.send(), "Could not send Github API request")?;

    if !res.status().is_success() {
        let err = catch(res.text(), "Could not display response text")?;

        return Err(CollarError::new(format!("Github API request failed: {err}")).boxed());
    }

    let text = catch(res.text(), "Could not extract response text")?;

    let releases: GithubApiReleasesResponse = catch(
        serde_json::from_str(&text),
        "Could not parse Github API response"
    )?;

    let latest = releases.0.get(0).ok_or(
        CollarError::new("Could not find latest release").boxed()
    )?;

    Ok(latest.tag_name.0)

}

fn get_file_url(version: Version, filename: impl AsRef<str>) -> String {
    "https://".to_string() + RELEASES_URL + "/download/" + "v" + &version.to_string() + "/" + filename.as_ref()
}

fn get_otrc_file_url(version: Version) -> String {
    let platform = CURRENT_PLATFORM;

    let mut filename = "otrc-".to_string() + platform;

    if cfg!(windows) {
        filename += ".exe";
    }

    get_file_url(version, filename)
}

fn get_otrrun_file_url(version: Version) -> String {
    let platform = CURRENT_PLATFORM;

    let mut filename = "otrrun-".to_string() + platform;

    if cfg!(windows) {
        filename += ".exe";
    }

    get_file_url(version, filename)
}

fn download_file(url: String) -> Result<Vec<u8>> {

    let response = catch(reqwest::blocking::get(&url), "Download request could not be sent")?;
    
    if !response.status().is_success() {
        let err_str = response.text().unwrap_or("Could not display response".to_string());

        return Err(CollarError::new(format!("Could not download the requested file: {err_str}")).boxed());
    }

    let bytes = catch(response.bytes(), "Invalid response")?;

    Ok(bytes.to_vec())
}

pub fn download_otrc(version: Option<Version>) -> Result<(Version, Vec<u8>)> {

    let version = version.unwrap_or(fetch_latest_release_version()?);

    let url = get_otrc_file_url(version);

    println!("Getting version {} from {url}", (&version.to_string() as &str).blue());

    let bytes = download_file(url)?;

    Ok((version, bytes))
}

pub fn download_otrrun(version: Option<Version>) -> Result<(Version, Vec<u8>)> {

    let version = version.unwrap_or(fetch_latest_release_version()?);

    let url = get_otrrun_file_url(version);

    println!("Getting version {} from {url}", (&version.to_string() as &str).blue());

    let bytes = download_file(url)?;

    Ok((version, bytes))
}