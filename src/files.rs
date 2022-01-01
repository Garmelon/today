use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDate, Utc};
use tzfile::Tz;

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
    pub fn new(file: usize, command: usize) -> Self {
        Self { file, command }
    }

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
    logs: HashMap<NaiveDate, Source>,
}

impl Files {
    /* Loading */

    pub fn load(path: &Path) -> Result<Self> {
        // Track already loaded files by their normalized paths
        let mut loaded = HashSet::new();

        let mut files = vec![];
        Self::load_file(&mut loaded, &mut files, path)?;

        let timezone = Self::determine_timezone(&files)?;
        let logs = Self::collect_logs(&files)?;
        Ok(Self {
            files,
            timezone,
            logs,
        })
    }

    fn load_file(
        loaded: &mut HashSet<PathBuf>,
        files: &mut Vec<LoadedFile>,
        name: &Path,
    ) -> Result<()> {
        let path = name.canonicalize().map_err(|e| Error::ResolvePath {
            path: name.to_path_buf(),
            error: e,
        })?;
        if loaded.contains(&path) {
            // We've already loaded this exact file.
            return Ok(());
        }

        let content = fs::read_to_string(name).map_err(|e| Error::ReadFile {
            file: path.clone(),
            error: e,
        })?;

        // Using `name` instead of `path` for the unwrap below.
        let file = parse::parse(name, &content)?;

        let includes = file
            .commands
            .iter()
            .filter_map(|c| match c {
                Command::Include(path) => Some(path.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        loaded.insert(path.clone());
        files.push(LoadedFile::new(path, name.to_owned(), file));

        for include in includes {
            // Since we've successfully opened the file, its name can't be the
            // root directory or empty string and it must thus have a parent.
            let include_path = name.parent().unwrap().join(include);
            Self::load_file(loaded, files, &include_path)?;
        }

        Ok(())
    }

    fn determine_timezone(files: &[LoadedFile]) -> Result<Tz> {
        let mut found: Option<String> = None;

        for command in Self::commands_of_files(files) {
            if let Command::Timezone(tz) = command.command {
                if let Some(found_tz) = &found {
                    if tz != found_tz {
                        return Err(Error::TzConflict {
                            tz1: found_tz.clone(),
                            tz2: tz.clone(),
                        });
                    }
                } else {
                    found = Some(tz.clone());
                }
            }
        }

        Ok(if let Some(timezone) = found {
            Tz::named(&timezone).map_err(|error| Error::ResolveTz { timezone, error })?
        } else {
            Tz::local().map_err(|error| Error::LocalTz { error })?
        })
    }

    fn collect_logs(files: &[LoadedFile]) -> Result<HashMap<NaiveDate, Source>> {
        let mut logs = HashMap::new();

        for command in Self::commands_of_files(files) {
            if let Command::Log(log) = command.command {
                if let Entry::Vacant(e) = logs.entry(log.date) {
                    e.insert(command.source);
                } else {
                    return Err(Error::LogConflict(log.date));
                }
            }
        }

        Ok(logs)
    }

    /* Saving */

    pub fn save(&self) -> Result<()> {
        for file in &self.files {
            if file.dirty {
                Self::save_file(&file.path, &file.file)?;
            }
        }
        Ok(())
    }

    fn save_file(path: &Path, file: &File) -> Result<()> {
        // TODO Sort commands within file
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

    /* Querying */

    pub fn files(&self) -> Vec<(&Path, &File)> {
        self.files
            .iter()
            .map(|f| (&f.name as &Path, &f.file))
            .collect()
    }

    fn commands_of_files(files: &[LoadedFile]) -> Vec<SourcedCommand<'_>> {
        let mut result = vec![];
        for (file_index, file) in files.iter().enumerate() {
            for (command_index, command) in file.file.commands.iter().enumerate() {
                let source = Source::new(file_index, command_index);
                result.push(SourcedCommand { source, command });
            }
        }
        result
    }

    pub fn commands(&self) -> Vec<SourcedCommand<'_>> {
        Self::commands_of_files(&self.files)
    }

    pub fn command(&self, source: Source) -> &Command {
        &self.files[source.file].file.commands[source.command]
    }

    pub fn now(&self) -> DateTime<&Tz> {
        Utc::now().with_timezone(&&self.timezone)
    }

    /* Updating */

    pub fn mark_all_dirty(&mut self) {
        for file in self.files.iter_mut() {
            file.dirty = true;
        }
    }

    /*
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
    */
}
