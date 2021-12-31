use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use tzfile::Tz;

use crate::eval::SourceInfo;

use self::commands::{Command, Done, File};
pub use self::error::{Error, Result};

pub mod arguments;
pub mod commands;
mod error;
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
        let formatted = format!("{}", file);
        if file.contents == formatted {
            println!("Unchanged file {:?}", path);
        } else {
            println!("Saving file {:?}", path);
            fs::write(path, &formatted).map_err(|e| Error::WriteFile {
                file: path.to_path_buf(),
                error: e,
            })?;
        }
        Ok(())
    }

    pub fn mark_all_dirty(&mut self) {
        for file in self.files.iter_mut() {
            file.dirty = true;
        }
    }

    pub fn command(&self, source: Source) -> &Command {
        &self.files[source.file].file.commands[source.command]
    }

    pub fn sources(&self) -> Vec<SourceInfo<'_>> {
        self.files
            .iter()
            .map(|f| SourceInfo {
                name: Some(f.name.to_string_lossy().to_string()),
                content: &f.file.contents,
            })
            .collect()
    }

    /// Add a [`Done`] statement to the task identified by `source`.
    ///
    /// Returns whether the addition was successful. It can fail if the entry
    /// identified by `source` is a note, not a task.
    #[must_use]
    pub fn add_done(&mut self, source: Source, done: Done) -> bool {
        let file = &mut self.files[source.file];
        match &mut file.file.commands[source.command] {
            Command::Task(t) => t.done.push(done),
            Command::Note(_) => return false,
        }
        file.dirty = true;
        true
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
