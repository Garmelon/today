use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::Files;
use codespan_reporting::term::{self, Config};
use termcolor::StandardStream;

pub trait Eprint<'a, F: Files<'a>> {
    fn eprint_diagnostic<'f: 'a>(
        files: &'f F,
        config: &Config,
        diagnostic: &Diagnostic<F::FileId>,
    ) {
        let mut out = StandardStream::stderr(termcolor::ColorChoice::Auto);
        if let Err(e) = term::emit(&mut out, config, files, diagnostic) {
            panic!("Error while reporting error: {}", e);
        }
    }

    fn eprint<'f: 'a>(&self, files: &'f F, config: &Config);
}

pub fn eprint_error<'a, 'f: 'a, F, E>(files: &'f F, e: &E)
where
    F: Files<'a>,
    E: Eprint<'a, F>,
{
    let config = Config::default();
    e.eprint(files, &config);
}
