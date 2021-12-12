use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io, result};

use chrono::{DateTime, Utc};
use tzfile::Tz;

use self::commands::{Command, File};

pub mod commands;
mod format;
mod parse;
pub mod primitives;

#[derive(Debug)]
struct LoadedFile {
    /// Canonical path for this file
    path: PathBuf,
    // User-readable path for this file
    name: PathBuf,
    file: File,
    /// Whether this file has been changed
    dirty: bool,
}

impl LoadedFile {
    pub fn new(path: PathBuf, name: PathBuf, file: File) -> Self {
        Self {
            path,
            name,
            file,
            dirty: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Source {
    file: usize,
    command: usize,
}

impl Source {
    pub fn file(&self) -> usize {
        self.file
    }
}

#[derive(Debug)]
pub struct SourcedCommand<'a> {
    pub source: Source,
    pub command: &'a Command,
}

#[derive(Debug)]
pub struct Files {
    files: Vec<LoadedFile>,
    timezone: Tz,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not resolve {path}: {error}")]
    ResolvePath { path: PathBuf, error: io::Error },
    #[error("Could not load {file}: {error}")]
    ReadFile { file: PathBuf, error: io::Error },
    #[error("Could not write {file}: {error}")]
    WriteFile { file: PathBuf, error: io::Error },
    #[error("Could not resolve timezone {timezone}: {error}")]
    ResolveTz { timezone: String, error: io::Error },
    #[error("Could not determine local timezone: {error}")]
    LocalTz { error: io::Error },
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
        let mut paths = HashMap::new();
        let mut files = vec![];
        Self::load_file(&mut paths, &mut files, path)?;
        let timezone = Self::determine_timezone(&files)?;
        Ok(Self { files, timezone })
    }

    fn load_file(
        paths: &mut HashMap<PathBuf, usize>,
        files: &mut Vec<LoadedFile>,
        name: &Path,
    ) -> Result<()> {
        let path = name.canonicalize().map_err(|e| Error::ResolvePath {
            path: name.to_path_buf(),
            error: e,
        })?;
        if paths.contains_key(&path) {
            // We've already loaded this exact file.
            return Ok(());
        }

        let content = fs::read_to_string(name).map_err(|e| Error::ReadFile {
            file: path.clone(),
            error: e,
        })?;

        // Using `name` instead of `path` for the unwrap below.
        let file = parse::parse(name, &content)?;
        let includes = file.includes.clone();

        paths.insert(path.clone(), files.len());
        files.push(LoadedFile::new(path, name.to_owned(), file));

        for include in includes {
            // Since we've successfully opened the file, its name can't be the
            // root directory or empty string and must thus have a parent.
            let include_path = name.parent().unwrap().join(include);
            Self::load_file(paths, files, &include_path)?;
        }

        Ok(())
    }

    fn determine_timezone(files: &[LoadedFile]) -> Result<Tz> {
        let mut found: Option<(PathBuf, String)> = None;

        for file in files {
            if let Some(file_tz) = &file.file.timezone {
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
            Tz::named(&tz).map_err(|e| Error::ResolveTz {
                timezone: tz,
                error: e,
            })?
        } else {
            Tz::local().map_err(|e| Error::LocalTz { error: e })?
        })
    }

    pub fn save(&self) -> Result<()> {
        for file in &self.files {
            if file.dirty {
                Self::save_file(&file.path, &file.file)?;
            }
        }
        Ok(())
    }

    fn save_file(path: &Path, file: &File) -> Result<()> {
        fs::write(path, &format!("{}", file)).map_err(|e| Error::WriteFile {
            file: path.to_path_buf(),
            error: e,
        })?;
        Ok(())
    }

    pub fn mark_main_dirty(&mut self) {
        if let Some(file) = self.files.get_mut(0) {
            file.dirty = true;
        }
    }

    pub fn mark_all_dirty(&mut self) {
        for file in self.files.iter_mut() {
            file.dirty = true;
        }
    }

    pub fn command(&self, source: Source) -> &Command {
        &self.files[source.file].file.commands[source.command]
    }

    pub fn commands(&self) -> Vec<SourcedCommand<'_>> {
        let mut result = vec![];
        for (file_index, file) in self.files.iter().enumerate() {
            for (command_index, command) in file.file.commands.iter().enumerate() {
                let source = Source {
                    file: file_index,
                    command: command_index,
                };
                result.push(SourcedCommand { source, command });
            }
        }
        result
    }

    pub fn now(&self) -> DateTime<&Tz> {
        Utc::now().with_timezone(&&self.timezone)
    }
}
