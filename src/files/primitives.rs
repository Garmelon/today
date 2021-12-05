use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl<'a> From<&pest::Span<'a>> for Span {
    fn from(pspan: &pest::Span<'a>) -> Self {
        Self {
            start: pspan.start(),
            end: pspan.end(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl<T: fmt::Debug> fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.value.fmt(f)
    }
}

impl<T> Spanned<T> {
    pub fn new(span: Span, value: T) -> Self {
        Self { span, value }
    }
}

// I don't know how one would write this. It works as a polymorphic standalone
// function, but not in an impl block.
// impl<S, T: Into<S>> Spanned<T> {
//     pub fn convert(&self) -> Spanned<S> {
//         Self::new(self.span, self.value.into())
//     }
// }

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time {
    pub hour: u8,
    pub min: u8,
}

impl fmt::Debug for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.min)
    }
}

impl Time {
    pub fn new(hour: u32, min: u32) -> Option<Self> {
        if hour < 24 && min < 60 || hour == 24 && min == 0 {
            Some(Self {
                hour: hour as u8,
                min: min as u8,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<chrono::Weekday> for Weekday {
    fn from(wd: chrono::Weekday) -> Self {
        match wd {
            chrono::Weekday::Mon => Self::Monday,
            chrono::Weekday::Tue => Self::Tuesday,
            chrono::Weekday::Wed => Self::Wednesday,
            chrono::Weekday::Thu => Self::Thursday,
            chrono::Weekday::Fri => Self::Friday,
            chrono::Weekday::Sat => Self::Saturday,
            chrono::Weekday::Sun => Self::Sunday,
        }
    }
}

impl Weekday {
    pub fn name(&self) -> &'static str {
        match self {
            Weekday::Monday => "mon",
            Weekday::Tuesday => "tue",
            Weekday::Wednesday => "wed",
            Weekday::Thursday => "thu",
            Weekday::Friday => "fri",
            Weekday::Saturday => "sat",
            Weekday::Sunday => "sun",
        }
    }

    pub fn num(&self) -> u8 {
        match self {
            Weekday::Monday => 1,
            Weekday::Tuesday => 2,
            Weekday::Wednesday => 3,
            Weekday::Thursday => 4,
            Weekday::Friday => 5,
            Weekday::Saturday => 6,
            Weekday::Sunday => 7,
        }
    }

    /// How many days from now until the other weekday.
    pub fn until(&self, other: Self) -> u8 {
        let num_self = self.num();
        let num_other = other.num();
        if num_self <= num_other {
            num_other - num_self
        } else {
            num_other + 7 - num_self
        }
    }
}
