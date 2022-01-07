use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::{fs, result};

use chrono::{DateTime, NaiveDate, Utc};
use codespan_reporting::files::SimpleFiles;
use tzfile::Tz;

use self::commands::{Command, Done, File, Log};
pub use self::error::{Error, ParseError, Result};
use self::primitives::Spanned;

pub mod arguments;
pub mod commands;
mod error;
mod format;
mod parse;
pub mod primitives;

// TODO Move file content from `File` to `LoadedFile`
#[derive(Debug)]
struct LoadedFile {
    /// User-readable path for this file.
    name: PathBuf,
    /// Identifier for codespan-reporting.
    cs_id: usize,
    file: File,
    /// Whether this file has been changed.
    dirty: bool,
}

impl LoadedFile {
    pub fn new(name: PathBuf, cs_id: usize, file: File) -> Self {
        Self {
            name,
            cs_id,
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

// TODO Rename to `SourceFile`?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileSource(usize);

impl Source {
    pub fn new(file: usize, command: usize) -> Self {
        Self { file, command }
    }

    pub fn file(&self) -> FileSource {
        FileSource(self.file)
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
    /// Codespan-reporting file database.
    cs_files: SimpleFiles<String, String>,
    timezone: Option<Tz>,
    logs: HashMap<NaiveDate, Source>,
}

impl<'a> codespan_reporting::files::Files<'a> for Files {
    type FileId = FileSource;
    type Name = String;
    type Source = &'a str;

    fn name(
        &'a self,
        id: Self::FileId,
    ) -> result::Result<Self::Name, codespan_reporting::files::Error> {
        self.cs_files.name(self.cs_id(id))
    }

    fn source(
        &'a self,
        id: Self::FileId,
    ) -> result::Result<Self::Source, codespan_reporting::files::Error> {
        self.cs_files.source(self.cs_id(id))
    }

    fn line_index(
        &'a self,
        id: Self::FileId,
        byte_index: usize,
    ) -> result::Result<usize, codespan_reporting::files::Error> {
        self.cs_files.line_index(self.cs_id(id), byte_index)
    }

    fn line_range(
        &'a self,
        id: Self::FileId,
        line_index: usize,
    ) -> result::Result<std::ops::Range<usize>, codespan_reporting::files::Error> {
        self.cs_files.line_range(self.cs_id(id), line_index)
    }
}

impl Files {
    /* Loading */

    pub fn new() -> Self {
        Self {
            files: vec![],
            cs_files: SimpleFiles::new(),
            timezone: None,
            logs: HashMap::new(),
        }
    }

    /// Load a file and all its includes.
    ///
    /// # Warning
    ///
    /// - This function must be called before all other functions.
    /// - This function must only be called once.
    /// - If this function fails,
    ///   - it is safe to print the error with [`Files::eprint_diagnostic`] and
    ///   - no other function must be called.
    pub fn load(&mut self, path: &Path) -> Result<()> {
        if !self.files.is_empty() {
            panic!("Files::load called multiple times");
        }

        // Track already loaded files by their normalized paths
        let mut loaded = HashSet::new();

        self.load_file(&mut loaded, path)?;
        self.determine_timezone()?;
        self.collect_logs()?;

        Ok(())
    }

    fn load_file(&mut self, loaded: &mut HashSet<PathBuf>, name: &Path) -> Result<()> {
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
        let cs_id = self
            .cs_files
            .add(name.to_string_lossy().to_string(), content.clone());

        // Using `name` instead of `path` for the unwrap below.
        let file = match parse::parse(name, &content) {
            Ok(file) => file,
            Err(error) => {
                // Using a dummy file. This should be fine since we return an
                // error immediately after and the user must never call `load`
                // twice. Otherwise, we run the danger of overwriting a file
                // with empty content.
                self.files
                    .push(LoadedFile::new(name.to_owned(), cs_id, File::dummy()));
                return Err(Error::Parse {
                    file: FileSource(self.files.len() - 1),
                    error,
                });
            }
        };

        let includes = file
            .commands
            .iter()
            .filter_map(|c| match c {
                Command::Include(path) => Some(path.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        loaded.insert(path);
        self.files
            .push(LoadedFile::new(name.to_owned(), cs_id, file));

        for include in includes {
            // Since we've successfully opened the file, its name can't be the
            // root directory or empty string and it must thus have a parent.
            let include_path = name.parent().unwrap().join(include.value);
            self.load_file(loaded, &include_path)?;
        }

        Ok(())
    }

    fn determine_timezone(&mut self) -> Result<()> {
        assert_eq!(self.timezone, None);

        let mut found: Option<(Source, Spanned<String>)> = None;

        for command in self.commands() {
            if let Command::Timezone(tz) = command.command {
                if let Some((found_source, found_tz)) = &found {
                    if tz.value != found_tz.value {
                        return Err(Error::TzConflict {
                            file1: found_source.file(),
                            span1: found_tz.span,
                            tz1: found_tz.value.clone(),
                            file2: command.source.file(),
                            span2: tz.span,
                            tz2: tz.value.clone(),
                        });
                    }
                } else {
                    found = Some((command.source, tz.clone()));
                }
            }
        }

        let timezone = if let Some((source, tz)) = found {
            Tz::named(&tz.value).map_err(|error| Error::ResolveTz {
                file: source.file(),
                span: tz.span,
                tz: tz.value,
                error,
            })?
        } else {
            Tz::local().map_err(|error| Error::LocalTz { error })?
        };
        self.timezone = Some(timezone);

        Ok(())
    }

    fn collect_logs(&mut self) -> Result<()> {
        for command in Self::commands_of_files(&self.files) {
            if let Command::Log(log) = command.command {
                match self.logs.entry(log.date.value) {
                    Entry::Vacant(e) => {
                        e.insert(command.source);
                    }
                    Entry::Occupied(e) => {
                        let other_cmd = Self::command_of_files(&self.files, *e.get());
                        let other_span = match &other_cmd.command {
                            Command::Log(log) => log.date.span,
                            _ => unreachable!(),
                        };
                        return Err(Error::LogConflict {
                            file1: other_cmd.source.file(),
                            span1: other_span,
                            file2: command.source.file(),
                            span2: log.date.span,
                            date: log.date.value,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /* Saving */

    pub fn save(&self) -> Result<()> {
        for file in &self.files {
            if file.dirty {
                Self::save_file(file)?;
            }
        }
        Ok(())
    }

    fn save_file(file: &LoadedFile) -> Result<()> {
        // TODO Sort commands within file
        let formatted = format!("{}", file.file);
        if file.file.contents == formatted {
            println!("Unchanged file {:?}", file.name);
        } else {
            println!("Saving file {:?}", file.name);
            fs::write(&file.name, &formatted).map_err(|e| Error::WriteFile {
                file: file.name.to_path_buf(),
                error: e,
            })?;
        }
        Ok(())
    }

    /* Querying */

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

    fn command_of_files(files: &[LoadedFile], source: Source) -> SourcedCommand<'_> {
        let command = &files[source.file].file.commands[source.command];
        SourcedCommand { source, command }
    }

    pub fn command(&self, source: Source) -> SourcedCommand<'_> {
        Self::command_of_files(&self.files, source)
    }

    pub fn log(&self, date: NaiveDate) -> Option<&Log> {
        let source = *self.logs.get(&date)?;
        match self.command(source).command {
            Command::Log(log) => Some(log),
            _ => unreachable!(),
        }
    }

    pub fn now(&self) -> DateTime<&Tz> {
        if let Some(tz) = &self.timezone {
            Utc::now().with_timezone(&tz)
        } else {
            panic!("Called Files::now before Files::load");
        }
    }

    /* Updating */

    pub fn mark_all_dirty(&mut self) {
        for file in self.files.iter_mut() {
            file.dirty = true;
        }
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
            _ => return false,
        }
        file.dirty = true;
        true
    }

    /* Errors */

    fn cs_id(&self, file: FileSource) -> usize {
        self.files[file.0].cs_id
    }
}
