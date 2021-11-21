use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io, result};

use chrono_tz::Tz;

use self::commands::File;

pub mod commands;
mod format;
mod parse;

#[derive(Debug)]
pub struct Files {
    files: HashMap<PathBuf, File>,
    timezone: Option<Tz>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Parse(#[from] parse::Error),
    #[error("{file1} has time zone {tz1} but {file2} has time zone {tz2}")]
    TzConflict {
        file1: PathBuf,
        tz1: Tz,
        file2: PathBuf,
        tz2: Tz,
    },
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
        let includes = file.includes.clone();

        self.files.insert(canon_path, file);

        for include in includes {
            self.load_file(&include)?;
        }

        Ok(())
    }

    fn determine_timezone(&mut self) -> Result<()> {
        let mut found: Option<(PathBuf, Tz)> = None;

        for file in self.files.values() {
            if let Some(file_tz) = file.timezone {
                if let Some((found_name, found_tz)) = &found {
                    if *found_tz != file_tz {
                        return Err(Error::TzConflict {
                            file1: found_name.clone(),
                            tz1: *found_tz,
                            file2: file.name.clone(),
                            tz2: file_tz,
                        });
                    }
                } else {
                    found = Some((file.name.clone(), file_tz));
                }
            }
        }

        if let Some((_, tz)) = found {
            self.timezone = Some(tz);
        }

        Ok(())
    }
}
