use chrono::NaiveDate;

use crate::files::Files;

use super::error::Error;
use super::util;

pub fn log(files: &mut Files, date: NaiveDate) -> Result<(), Error> {
    let desc = files
        .log(date)
        .map(|log| log.value.desc.join("\n"))
        .unwrap_or_default();

    let edited = util::edit_with_suffix(&desc, ".md")?;

    let edited = edited
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    files.set_log(date, edited);

    Ok(())
}
