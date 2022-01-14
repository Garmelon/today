use chrono::NaiveDate;

use crate::files::Files;

use super::error::Error;

pub fn log(files: &mut Files, date: NaiveDate) -> Result<(), Error> {
    let desc = files
        .log(date)
        .map(|log| log.value.desc.join("\n"))
        .unwrap_or_default();

    let mut builder = edit::Builder::new();
    builder.suffix(".md");
    let edited = edit::edit_with_builder(desc, &builder)
        .map_err(|error| Error::EditingLog { date, error })?;

    let edited = edited
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    files.set_log(date, edited);

    Ok(())
}
