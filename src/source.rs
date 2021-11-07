use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::{fs, io};

use codespan_reporting::files::SimpleFiles;

#[derive(Debug, thiserror::Error)]
pub enum LoadFileError {
    #[error("file already loaded")]
    FileAlreadyLoaded,
    #[error("error loading file: {0}")]
    IoError(#[from] io::Error),
}

pub struct SourceFiles {
    files: SimpleFiles<String, String>,
    files_by_path: HashMap<PathBuf, usize>,
}

impl SourceFiles {
    pub fn new() -> Self {
        Self {
            files: SimpleFiles::new(),
            files_by_path: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: &Path) -> Result<SourceFile, LoadFileError> {
        let canonical_path = path.canonicalize()?;
        if self.files_by_path.contains_key(&canonical_path) {
            return Err(LoadFileError::FileAlreadyLoaded);
        }

        let name = path.as_os_str().to_string_lossy().into_owned();
        let content = fs::read_to_string(path)?;
        let file_id = self.files.add(name, content);
        Ok(SourceFile(file_id))
    }

    pub fn content_of(&self, file: SourceFile) -> Option<&str> {
        self.files.get(file.0).ok().map(|sf| sf.source() as &str)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SourceFile(usize);

impl SourceFile {
    pub fn span(&self, range: Range<usize>) -> SourceSpan {
        SourceSpan {
            file: self.0,
            start: range.start,
            end: range.end,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SourceSpan {
    file: usize,
    start: usize,
    end: usize,
}
