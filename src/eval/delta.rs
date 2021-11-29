use std::cmp::Ordering;

use crate::files::commands::{self, Spanned, Time, Weekday};

/// Like [`commands::DeltaStep`] but includes a new constructor,
/// [`DeltaStep::Time`].
#[derive(Debug, Clone, Copy)]
pub enum DeltaStep {
    Year(i32),
    Month(i32),
    MonthReverse(i32),
    Day(i32),
    Week(i32),
    Hour(i32),
    Minute(i32),
    Weekday(i32, Weekday),
    /// Set the time to the next occurrence of the specified time. Useful to
    /// unify the end delta and end time for different specs.
    Time(Time),
}

impl From<commands::DeltaStep> for DeltaStep {
    fn from(step: commands::DeltaStep) -> Self {
        match step {
            commands::DeltaStep::Year(n) => Self::Year(n),
            commands::DeltaStep::Month(n) => Self::Month(n),
            commands::DeltaStep::MonthReverse(n) => Self::MonthReverse(n),
            commands::DeltaStep::Day(n) => Self::Day(n),
            commands::DeltaStep::Week(n) => Self::Week(n),
            commands::DeltaStep::Hour(n) => Self::Hour(n),
            commands::DeltaStep::Minute(n) => Self::Minute(n),
            commands::DeltaStep::Weekday(n, wd) => Self::Weekday(n, wd),
        }
    }
}

impl DeltaStep {
    /// A lower bound on days
    fn lower_bound(&self) -> i32 {
        match self {
            DeltaStep::Year(n) => {
                if *n < 0 {
                    *n * 366
                } else {
                    *n * 365
                }
            }
            DeltaStep::Month(n) | DeltaStep::MonthReverse(n) => {
                if *n < 0 {
                    *n * 31
                } else {
                    *n * 28
                }
            }
            DeltaStep::Day(n) => *n,
            DeltaStep::Week(n) => *n * 7,
            DeltaStep::Hour(n) => {
                if *n < 0 {
                    *n / 24 + (*n % 24).signum()
                } else {
                    *n / 24
                }
            }
            DeltaStep::Minute(n) => {
                if *n < 0 {
                    *n / (24 * 60) + (*n % (24 * 60)).signum()
                } else {
                    *n / (24 * 60)
                }
            }
            DeltaStep::Weekday(n, _) => match n.cmp(&0) {
                Ordering::Less => *n * 7 - 1,
                Ordering::Equal => 0,
                Ordering::Greater => *n * 7 - 7,
            },
            DeltaStep::Time(_) => 0,
        }
    }

    /// An upper bound on days
    fn upper_bound(&self) -> i32 {
        match self {
            DeltaStep::Year(n) => {
                if *n > 0 {
                    *n * 366
                } else {
                    *n * 365
                }
            }
            DeltaStep::Month(n) | DeltaStep::MonthReverse(n) => {
                if *n > 0 {
                    *n * 31
                } else {
                    *n * 28
                }
            }
            DeltaStep::Day(n) => *n,
            DeltaStep::Week(n) => *n * 7,
            DeltaStep::Hour(n) => {
                if *n > 0 {
                    *n / 24 + (*n % 24).signum()
                } else {
                    *n / 24
                }
            }
            DeltaStep::Minute(n) => {
                if *n > 0 {
                    *n / (24 * 60) + (*n % (24 * 60)).signum()
                } else {
                    *n / (24 * 60)
                }
            }
            DeltaStep::Weekday(n, _) => match n.cmp(&0) {
                Ordering::Less => *n * 7 - 7,
                Ordering::Equal => 0,
                Ordering::Greater => *n * 7 - 1,
            },
            DeltaStep::Time(_) => 1,
        }
    }
}

#[derive(Debug, Default)]
pub struct Delta {
    pub steps: Vec<Spanned<DeltaStep>>,
}

impl From<&commands::Delta> for Delta {
    fn from(delta: &commands::Delta) -> Self {
        Self {
            steps: delta
                .0
                .iter()
                .map(|step| Spanned::new(step.span, step.value.into()))
                .collect(),
        }
    }
}

impl Delta {
    pub fn lower_bound(&self) -> i32 {
        self.steps.iter().map(|step| step.value.lower_bound()).sum()
    }

    pub fn upper_bound(&self) -> i32 {
        self.steps.iter().map(|step| step.value.upper_bound()).sum()
    }
}
