use std::str::FromStr;

use chrono::{NaiveDate, NaiveTime};
use nom::branch::Alt;
use nom::bytes::complete::{take_while1, take_while_m_n};
use nom::character::complete::{char, digit1, newline};
use nom::combinator::{eof, fail, map_opt, map_res};
use nom::sequence::terminated;
use nom::{IResult, Parser};

fn line_ending(i: &str) -> IResult<&str, ()> {
    (newline.map(|_| ()), eof.map(|_| ())).choice(i)
}

fn title(i: &str) -> IResult<&str, &str> {
    terminated(take_while1(|c| c != '\n'), line_ending)(i)
}

fn number<N: FromStr>(i: &str) -> IResult<&str, N> {
    map_res(digit1, str::parse)(i)
}

fn fixed_width_number<N: FromStr, const W: usize>(i: &str) -> IResult<&str, N> {
    map_res(
        take_while_m_n(W, W, |c: char| c.is_ascii_digit()),
        str::parse,
    )(i)
}

fn date(i: &str) -> IResult<&str, NaiveDate> {
    let (i, year) = number::<i32>(i)?;
    let (i, _) = char('-')(i)?;
    let (i, month) = fixed_width_number::<u32, 2>(i)?;
    let (i, _) = char('-')(i)?;
    let (i, day) = fixed_width_number::<u32, 2>(i)?;
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => Ok((i, date)),
        None => fail(i),
    }
}

fn time(i: &str) -> IResult<&str, NaiveTime> {
    let (i, hour) = fixed_width_number::<u32, 2>(i)?;
    let (i, _) = char(':')(i)?;
    let (i, min) = fixed_width_number::<u32, 2>(i)?;
    if hour < 24 && min < 60 {
        Ok((i, NaiveTime::from_hms(hour, min, 0)))
    } else {
        fail(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_ending() {
        assert_eq!(line_ending("\n"), Ok(("", ())));
        assert_eq!(line_ending("\nbla"), Ok(("bla", ())));
        assert_eq!(line_ending("\n\n"), Ok(("\n", ())));
        assert_eq!(line_ending(""), Ok(("", ())));

        assert!(line_ending("bla").is_err());
        assert!(line_ending("\r").is_err());
        assert!(line_ending("\r\n").is_err());
    }

    #[test]
    fn test_title() {
        assert_eq!(title("foo bar\nbaz"), Ok(("baz", "foo bar")));
        assert_eq!(title("foo bar\n"), Ok(("", "foo bar")));
        assert_eq!(title("foo bar"), Ok(("", "foo bar")));
        assert_eq!(title(" \nbla"), Ok(("bla", " ")));
        assert_eq!(
            title("!\"§$%&/()'<>[]{}_:;.,-=\nbla"),
            Ok(("bla", "!\"§$%&/()'<>[]{}_:;.,-="))
        );

        assert!(title("\nxyz").is_err());
        assert!(title("").is_err());
    }

    #[test]
    fn test_number() {
        assert_eq!(number::<u32>("012345abc"), Ok(("abc", 12345)));
        assert_eq!(number::<u8>("255"), Ok(("", 255)));
        assert_eq!(number::<i64>("0x3f"), Ok(("x3f", 0)));

        assert!(number::<u8>("256").is_err());
        assert!(number::<i32>("xyz").is_err());
        assert!(number::<i8>("").is_err());
    }

    #[test]
    fn test_fixed_width_number() {
        assert_eq!(
            fixed_width_number::<u32, 2>("012345abc"),
            Ok(("2345abc", 1))
        );
        assert_eq!(
            fixed_width_number::<u32, 3>("012345abc"),
            Ok(("345abc", 12))
        );
        assert_eq!(
            fixed_width_number::<u32, 6>("012345abc"),
            Ok(("abc", 12345))
        );

        assert!(fixed_width_number::<u8, 0>("012345abc").is_err());
        assert!(fixed_width_number::<u8, 6>("012345abc").is_err());
        assert!(fixed_width_number::<u8, 3>("14x").is_err());
        assert!(fixed_width_number::<i8, 4>("").is_err());
    }

    #[test]
    fn test_date() {
        assert_eq!(
            date("2021-11-06"),
            Ok(("", NaiveDate::from_ymd(2021, 11, 6)))
        );
        assert_eq!(
            date("2021-11-0678"),
            Ok(("78", NaiveDate::from_ymd(2021, 11, 6)))
        );
        assert_eq!(date("0-01-01"), Ok(("", NaiveDate::from_ymd(0, 1, 1))));
        assert_eq!(
            date("2020-02-29"),
            Ok(("", NaiveDate::from_ymd(2020, 2, 29)))
        );

        assert!(date("2021-11-6").is_err());
        assert!(date("0000-00-00").is_err());
        assert!(date("2021-02-29").is_err());
    }

    #[test]
    fn test_time() {
        assert_eq!(time("12:34"), Ok(("", NaiveTime::from_hms(12, 34, 0))));
        assert_eq!(time("00:00"), Ok(("", NaiveTime::from_hms(0, 0, 0))));
        assert_eq!(time("23:59"), Ok(("", NaiveTime::from_hms(23, 59, 0))));
        assert_eq!(time("02:04:06"), Ok((":06", NaiveTime::from_hms(2, 4, 0))));

        assert!(time("abc").is_err());
        assert!(time("24:23").is_err());
        assert!(time("12:60").is_err());
        assert!(time("12-34").is_err());
        assert!(time("2:34").is_err());
        assert!(time("12:3").is_err());
    }
}
