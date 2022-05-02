# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## Unreleased

### Changed
- Birthdays for current day are now highlighted

## 0.2.0 - 2022-03-18

### Added
- `LOG` command and `today log` CLI command
- `CAPTURE` command and `today new` CLI command
- `REMIND` statement
- `CANCEL` statement for tasks
- One-letter aliases for `show`, `log`, `done` and `cancel` CLI commands
- `MOVE` can now move entries to a different time
- `--date` now accepts expressions like `today-3d`
- In `--range` and `--date`, `t` can be used as abbreviation for `today`
- `*` markers in output for days with logs and entries with descriptions

### Changed
- Output is now colored
- Better error messages
- Overhauled `today show` format
    - It can now show log entries for days
    - It now displays the source command (file and line) of the entry
- When saving...
    - Unchanged files are no longer overwritten
    - Imports are now sorted alphabetically
    - Done and cancel dates are now simplified where possible
- Always prints import-based path, not absolute path

### Fixed
- Alignment in output
- Respect `TZDIR` environment variable
- Negative weekday deltas

## 0.1.0 - 2021-12-20

### Added
- Initial implementation, including...
- Parsing
    - `INCLUDE`, `TIMEZONE`, `NOTE` and `TASK` commands
    - `DATE`, `BDATE`, `FROM`, `UNTIL`, `MOVE` and `EXCEPT` statements
- CLI
    - `--file`, `--date` and `--range` arguments
    - `show`, `done` and `fmt` commands
- Readme
- This changelog
- Example file
- Git pre-commit hook ensuring code is formatted
