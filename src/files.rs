use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io, result};

use chrono::{DateTime, Utc};
use tzfile::Tz;

use self::commands::File;

pub mod commands;
mod format;
mod parse;

#[derive(Debug)]
pub struct Files {
    files: HashMap<PathBuf, File>,
    timezone: Tz,
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
        tz1: String,
        file2: PathBuf,
        tz2: String,
    },
}

pub type Result<T> = result::Result<T, Error>;

impl Files {
    pub fn load(path: &Path) -> Result<Self> {
        let mut files = HashMap::new();
        Self::load_file(&mut files, path)?;
        let timezone = Self::determine_timezone(&files)?;
        Ok(Self { files, timezone })
    }

    fn load_file(files: &mut HashMap<PathBuf, File>, path: &Path) -> Result<()> {
        let canon_path = path.canonicalize()?;
        if files.contains_key(&canon_path) {
            // We've already loaded this exact file.
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        let file = parse::parse(path, &content)?;
        let includes = file.includes.clone();

        files.insert(canon_path, file);

        for include in includes {
            Self::load_file(files, &include)?;
        }

        Ok(())
    }

    fn determine_timezone(files: &HashMap<PathBuf, File>) -> Result<Tz> {
        let mut found: Option<(PathBuf, String)> = None;

        for file in files.values() {
            if let Some(file_tz) = &file.timezone {
                if let Some((found_name, found_tz)) = &found {
                    if found_tz != file_tz {
                        return Err(Error::TzConflict {
                            file1: found_name.clone(),
                            tz1: found_tz.clone(),
                            file2: file.name.clone(),
                            tz2: file_tz.clone(),
                        });
                    }
                } else {
                    found = Some((file.name.clone(), file_tz.clone()));
                }
            }
        }

        Ok(if let Some((_, tz)) = found {
            Tz::named(&tz)?
        } else {
            Tz::local()?
        })
    }

    pub fn now(&self) -> DateTime<&Tz> {
        Utc::now().with_timezone(&&self.timezone)
    }
}
