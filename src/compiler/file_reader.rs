use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    compiler::CompilerError,
    lexer::{
        fragmenter::FragmentStream,
        token::{ContextualizedToken, ContextualizedTokenStream, Token, TokenStream},
        Tokenizer,
    },
};

use crate::error::Result;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ImportAddress {
    pub module_id: String,
    pub path: Option<String>,
}

impl Display for ImportAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}",
            self.path.as_ref().unwrap_or(&("".to_string())),
            self.module_id
        )
    }
}

struct SourceFile {
    path: PathBuf,
    tokens: Box<dyn Iterator<Item = ContextualizedToken>>,
}

impl Iterator for SourceFile {
    type Item = ContextualizedToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.next()
    }
}

pub struct FileReader {
    root_file_path: PathBuf,
    source_stack: Vec<SourceFile>,
    read_modules: HashSet<ImportAddress>,

    tokenizer: Tokenizer,
}

impl FileReader {
    pub fn new(tokenizer: Tokenizer, root_file_path: PathBuf) -> Self {
        Self {
            root_file_path,

            source_stack: Vec::new(),
            read_modules: HashSet::new(),

            tokenizer,
        }
    }

    fn resolve_path(&self, import_address: &ImportAddress) -> Result<PathBuf> {
        let mut path = self.root_file_path.clone();

        if let Some(location) = &import_address.path {
            path = path.join(location);
        }
        path = path.join(import_address.module_id.clone() + ".otr");

        Ok(path)
    }

    pub fn try_read_module(&self, module: &ImportAddress) -> Result<String> {
        let path = self.resolve_path(module)?;

        fs::read_to_string(path).map_err(|err| {
            CompilerError::Unknown {
                message: format!(
                    "Module '{}' could not be loaded from the file system! {}",
                    module, err
                ),
            }
            .boxed()
        })
    }

    fn tokenize(&self, source: String) -> Result<ContextualizedTokenStream> {
        let fragments = FragmentStream::from_str(&source)?;

        self.tokenizer.tokenize(fragments)
    }

    pub fn push_dependency(&mut self, dependency: ImportAddress) -> Result<()> {
        if self.read_modules.contains(&dependency) {
            return Ok(());
        }

        let file = self.try_read_module(&dependency)?;
        let tokens = self.tokenize(file)?;
        self.source_stack.push(SourceFile {
            path: self.resolve_path(&dependency)?,
            tokens: Box::new(tokens.into_iter()),
        });

        self.read_modules.insert(dependency);

        Ok(())
    }

    pub fn get_current_file(&self) -> Result<&Path> {
        Ok(&self
            .source_stack
            .last()
            .ok_or(
                CompilerError::Unknown {
                    message: "No current file in file reader!".into(),
                }
                .boxed(),
            )?
            .path)
    }
}

impl Iterator for FileReader {
    type Item = ContextualizedToken;

    fn next(&mut self) -> Option<Self::Item> {
        for i in (0..self.source_stack.len()).rev() {
            if let Some(token) = self.source_stack[i].next() {
                return Some(token);
            } else {
                self.source_stack.pop();
            }
        }

        None
    }
}
