use std::{collections::{HashSet, VecDeque}, fmt::Display, fs, path::{Path, PathBuf}, str::FromStr};

use crate::{compiler::CompilerError, lexer::{FragmentStream, token::Token}};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ImportAddress {
    pub module_id: String,
    pub path: Option<String>,
}

impl Display for ImportAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.path.as_ref().unwrap_or(&("".to_string())), self.module_id)
    }
}

pub struct FileReader {
    root_file_path: PathBuf,
    queue: VecDeque<ImportAddress>,
    read_modules: HashSet<ImportAddress>
}

impl FileReader {
    pub fn new(root_file_path: PathBuf) -> Self {
        Self {
            root_file_path,

            queue: VecDeque::new(),
            read_modules: HashSet::new(),
        }
    }

    pub fn try_read_module(&self, module: &ImportAddress) -> Result<String, CompilerError> {
        let mut path = self.root_file_path.clone();
        
            if let Some(location) = &module.path {
                path = path.join(location);
            }
            path = path.join(module.module_id.clone() + ".otr");

        fs::read_to_string(path).map_err(|err| CompilerError {
            message: format!("Module '{}' could not be loaded from the file system! {}", module, err)
        })
    }

    pub fn enqueue(&mut self, module: ImportAddress) {
        if !self.read_modules.contains(&module) {
            self.queue.push_back(module.clone());
            self.read_modules.insert(module);
        }
    }

    pub fn dequeue(&mut self) -> Result<Option<String>, CompilerError> {
        if self.queue.is_empty() {
            return Ok(None);
        }

        let module = self.queue.pop_front().unwrap();

        Ok(Some(self.try_read_module(&module)?))
    }
}