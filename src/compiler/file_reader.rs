use std::{collections::{HashSet, VecDeque}, fs, path::{Path, PathBuf}, str::FromStr};

use crate::{compiler::CompilerError, lexer::{FragmentStream, token::Token}};


pub struct FileReader {
    root_file_path: PathBuf,
    queue: VecDeque<String>,
    read_modules: HashSet<String>
}

impl FileReader {
    pub fn new(root_file_path: PathBuf) -> Self {
        Self {
            root_file_path,

            queue: VecDeque::new(),
            read_modules: HashSet::new(),
        }
    }

    pub fn try_read_module(&self, module: &String) -> Result<String, CompilerError> {
        let path = self.root_file_path.join(module.clone() + ".otr");

        fs::read_to_string(path).map_err(|err| CompilerError {
            message: format!("Module '{}' could not be loaded from the file system! {}", module, err)
        })
    }

    pub fn enqueue(&mut self, module: String) {
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