use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io, result};

use chrono_tz::Tz;

use crate::commands::File;
use crate::parse;

#[derive(Debug)]
pub struct Files {
    files: HashMap<PathBuf, File>,
    timezone: Option<Tz>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("{0}")]
    ParseError(#[from] parse::Error),
}

pub type Result<T> = result::Result<T, Error>;

impl Files {
    pub fn load(path: &Path) -> Result<Self> {
        let mut new = Self {
            files: HashMap::new(),
            timezone: None,
        };

        new.load_file(path)?;
        new.determine_timezone()?;

        Ok(new)
    }

    fn load_file(&mut self, path: &Path) -> Result<()> {
        let canon_path = path.canonicalize()?;
        if self.files.contains_key(&canon_path) {
            // We've already loaded this exact file.
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        let file = parse::parse(path, &content)?;
        self.files.insert(canon_path, file);

        // TODO Also load all imported files

        Ok(())
    }

    fn determine_timezone(&mut self) -> Result<()> {
        // TODO Implement once files can specify time zones
        Ok(())
    }
}
