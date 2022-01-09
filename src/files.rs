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
    /// Commands that have been removed and are to be skipped during formatting.
    ///
    /// They are not directly removed from the list of commands in order not to
    /// change other commands' indices.
    removed: HashSet<usize>,
}

impl LoadedFile {
    pub fn new(name: PathBuf, cs_id: usize, file: File) -> Self {
        Self {
            name,
            cs_id,
            file,
            dirty: false,
            removed: HashSet::new(),
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
pub struct Sourced<'a, T> {
    pub source: Source,
    pub value: &'a T,
}

impl<'a, T> Sourced<'a, T> {
    fn new(source: Source, value: &'a T) -> Self {
        Self { source, value }
    }
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
    ///   - it is safe to print the error using the [`codespan_reporting::files::Files`] instance and
    ///   - no other functions may be called.
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
            .filter_map(|c| match &c.value {
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
            if let Command::Timezone(tz) = &command.value.value {
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
            if let Command::Log(log) = &command.value.value {
                match self.logs.entry(log.date.value) {
                    Entry::Vacant(e) => {
                        e.insert(command.source);
                    }
                    Entry::Occupied(e) => {
                        let other_cmd = Self::command_of_files(&self.files, *e.get());
                        let other_span = match &other_cmd.value.value {
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
                self.save_file(file)?;
            }
        }
        Ok(())
    }

    fn save_file(&self, file: &LoadedFile) -> Result<()> {
        // TODO Sort commands within file

        let previous = self
            .cs_files
            .get(file.cs_id)
            .expect("cs id is valid")
            .source();

        let formatted = file.file.format(&file.removed);

        if previous == &formatted {
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

    fn commands_of_files(files: &[LoadedFile]) -> Vec<Sourced<'_, Spanned<Command>>> {
        let mut result = vec![];
        for (file_index, file) in files.iter().enumerate() {
            for (command_index, command) in file.file.commands.iter().enumerate() {
                let source = Source::new(file_index, command_index);
                result.push(Sourced::new(source, command));
            }
        }
        result
    }

    pub fn commands(&self) -> Vec<Sourced<'_, Spanned<Command>>> {
        Self::commands_of_files(&self.files)
    }

    fn command_of_files(files: &[LoadedFile], source: Source) -> Sourced<'_, Spanned<Command>> {
        let command = &files[source.file].file.commands[source.command];
        Sourced::new(source, command)
    }

    pub fn command(&self, source: Source) -> Sourced<'_, Spanned<Command>> {
        Self::command_of_files(&self.files, source)
    }

    pub fn log(&self, date: NaiveDate) -> Option<Sourced<'_, Log>> {
        let source = *self.logs.get(&date)?;
        match &self.command(source).value.value {
            Command::Log(log) => Some(Sourced::new(source, log)),
            _ => unreachable!(),
        }
    }

    fn latest_log(&self) -> Option<(NaiveDate, Source)> {
        self.logs
            .iter()
            .map(|(d, s)| (*d, *s))
            .max_by_key(|(d, _)| *d)
    }

    fn latest_log_before(&self, date: NaiveDate) -> Option<(NaiveDate, Source)> {
        self.logs
            .iter()
            .map(|(d, s)| (*d, *s))
            .filter(|(d, _)| d <= &date)
            .max_by_key(|(d, _)| *d)
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

    fn modify(&mut self, source: Source, edit: impl FnOnce(&mut Command)) {
        let file = &mut self.files[source.file];
        edit(&mut file.file.commands[source.command].value);
        file.dirty = true;
    }

    fn insert(&mut self, file: FileSource, command: Command) {
        let file = &mut self.files[file.0];
        file.file.commands.push(Spanned::dummy(command));
        file.dirty = true;
    }

    fn remove(&mut self, source: Source) {
        let file = &mut self.files[source.file];
        file.removed.insert(source.command);
        file.dirty = true;
    }

    /// Add a [`Done`] statement to the task identified by `source`.
    ///
    /// Returns whether the addition was successful. It can fail if the entry
    /// identified by `source` is a note, not a task.
    #[must_use]
    pub fn add_done(&mut self, source: Source, done: Done) -> bool {
        let file = &mut self.files[source.file];
        match &mut file.file.commands[source.command].value {
            Command::Task(t) => t.done.push(done),
            _ => return false,
        }
        file.dirty = true;
        true
    }

    pub fn set_log(&mut self, date: NaiveDate, desc: Vec<String>) {
        if let Some(source) = self.logs.get(&date).cloned() {
            if desc.is_empty() {
                self.remove(source);
            } else {
                self.modify(source, |command| match command {
                    Command::Log(log) => log.desc = desc,
                    _ => unreachable!(),
                });
            }
        } else if !desc.is_empty() {
            let file = self
                .latest_log_before(date)
                .or_else(|| self.latest_log())
                .map(|(_, source)| source.file())
                .unwrap_or(FileSource(0));

            let date = Spanned::dummy(date);
            let command = Command::Log(Log { date, desc });

            self.insert(file, command);
        }
    }

    /* Errors */

    fn cs_id(&self, file: FileSource) -> usize {
        self.files[file.0].cs_id
    }
}
