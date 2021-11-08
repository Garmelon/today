use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::{fs, io};

use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

#[derive(Debug)]
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

    pub fn load(&mut self, path: &Path) -> io::Result<(SourceFile, &str)> {
        let canonical_path = path.canonicalize()?;
        let file_id = if let Some(&file_id) = self.files_by_path.get(&canonical_path) {
            file_id
        } else {
            let name = path.as_os_str().to_string_lossy().into_owned();
            let content = fs::read_to_string(path)?;
            self.files.add(name, content)
        };

        let content = self.files.get(file_id).unwrap().source() as &str;
        Ok((SourceFile(file_id), content))
    }

    pub fn emit_all<'a, T>(&self, diagnostics: &'a [T])
    where
        &'a T: Into<Diagnostic<usize>>,
    {
        let stderr = StandardStream::stderr(ColorChoice::Auto);
        let config = term::Config::default();

        for diagnostic in diagnostics {
            let diagnostic: Diagnostic<usize> = diagnostic.into();
            term::emit(&mut stderr.lock(), &config, &self.files, &diagnostic)
                .expect("failed to print errors");
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SourceFile(usize);

impl SourceFile {
    pub fn span(&self, range: Range<usize>) -> SourceSpan {
        SourceSpan {
            file_id: self.0,
            start: range.start,
            end: range.end,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SourceSpan {
    file_id: usize,
    start: usize,
    end: usize,
}

impl SourceSpan {
    pub fn file_id(&self) -> usize {
        self.file_id
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}
