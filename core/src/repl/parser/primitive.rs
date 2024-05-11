//! Defines parser functions for the primitive types used in Ledger format.

use crate::repl::pretty_decimal::{self, PrettyDecimal};

use chrono::NaiveDate;
use winnow::{
    ascii::digit1,
    combinator::{alt, trace},
    error::{FromExternalError, ParserError},
    token::{one_of, take_till, take_while},
    PResult, Parser,
};

/// Parses comma separated decimal.
pub fn comma_decimal<'a, E>(input: &mut &'a str) -> PResult<PrettyDecimal, E>
where
    E: ParserError<&'a str> + FromExternalError<&'a str, pretty_decimal::Error>,
{
    trace(
        "primitive::comma_decimal",
        take_while(1.., |c: char| {
            c.is_ascii_digit() || c == '-' || c == ',' || c == '.'
        })
        .try_map(str::parse),
    )
    .parse_next(input)
}

const NON_COMMODITY_CHARS: [char; 37] = [
    ' ', '\t', '\r', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',', ';', ':',
    '?', '!', '-', '+', '*', '/', '^', '&', '|', '=', '<', '>', '[', ']', '(', ')', '{', '}', '@',
];

/// Parses commodity in greedy manner.
/// Returns empty string if the upcoming characters are not valid as commodity to support empty commodity.
pub fn commodity<'a, E: ParserError<&'a str>>(input: &mut &'a str) -> PResult<&'a str, E> {
    // Quoted commodity not supported.
    trace("primitive::commodity", take_till(0.., NON_COMMODITY_CHARS)).parse_next(input)
}

#[derive(Copy, Clone)]
enum DateType {
    Slash,
    Hyphen,
}

impl DateType {
    fn pattern(self) -> &'static str {
        match self {
            DateType::Slash => "%Y/%m/%d",
            DateType::Hyphen => "%F",
        }
    }
}

/// Parses date in yyyy/mm/dd format.
pub fn date<'a, E>(input: &mut &'a str) -> PResult<NaiveDate, E>
where
    E: ParserError<&'a str> + FromExternalError<&'a str, chrono::ParseError>,
{
    let slash = (digit1, one_of('/'), digit1, one_of('/'), digit1);
    let hyphen = (digit1, one_of('-'), digit1, one_of('-'), digit1);
    trace(
        "primitive::date",
        alt((slash.value(DateType::Slash), hyphen.value(DateType::Hyphen)))
            .with_recognized()
            .try_map(|(date_type, s)| NaiveDate::parse_from_str(s, date_type.pattern())),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;
    use winnow::error::{ErrMode, ErrorKind, InputError};

    #[test]
    fn comma_decimal_parses_valid_inputs() {
        assert_eq!(
            expect_parse_ok(comma_decimal, "123"),
            ("", PrettyDecimal::unformatted(dec!(123)))
        );
        assert_eq!(
            expect_parse_ok(comma_decimal, "-12,345.67 JPY"),
            (" JPY", PrettyDecimal::comma3dot(dec!(-12345.67)))
        );
        assert_eq!(
            expect_parse_ok(comma_decimal, "-012.3$"),
            ("$", PrettyDecimal::unformatted(dec!(-12.3)))
        );
    }

    #[test]
    fn comma_decimal_fails_on_invalid_inputs() {
        assert_eq!(
            comma_decimal.parse_peek("不可能"),
            Err(ErrMode::Backtrack(InputError::new(
                "不可能",
                ErrorKind::Slice
            )))
        );
        assert_eq!(
            comma_decimal.parse_peek("!"),
            Err(ErrMode::Backtrack(InputError::new("!", ErrorKind::Slice)))
        );
    }

    #[test]
    fn commodity_parses_valid_inputs() {
        assert_eq!(expect_parse_ok(commodity, "USD "), (" ", "USD"));
        assert_eq!(expect_parse_ok(commodity, "JPY\n"), ("\n", "JPY"));
        assert_eq!(expect_parse_ok(commodity, "$ $"), (" $", "$"));
        assert_eq!(expect_parse_ok(commodity, "£ "), (" ", "£"));
    }

    #[test]
    fn commodity_returns_empty_invalid() {
        assert_eq!(expect_parse_ok(commodity, "123"), ("123", ""));
        assert_eq!(expect_parse_ok(commodity, " "), (" ", ""));
    }

    #[test]
    fn date_parses_valid_inputs() {
        let res = expect_parse_ok(date, "2022/01/15");
        assert_eq!(res, ("", NaiveDate::from_ymd_opt(2022, 1, 15).unwrap()));

        let res = expect_parse_ok(date, "2022/2/3");
        assert_eq!(res, ("", NaiveDate::from_ymd_opt(2022, 2, 3).unwrap()));

        let res = expect_parse_ok(date, "2022-01-15");
        assert_eq!(res, ("", NaiveDate::from_ymd_opt(2022, 1, 15).unwrap()));
    }

    #[test]
    fn date_fails_on_invalid_inputs() {
        assert_eq!(
            date.parse_peek("not a date"),
            Err(ErrMode::Backtrack(InputError::new(
                "not a date",
                ErrorKind::Slice
            )))
        );
        assert_eq!(
            date.parse_peek("2022/01"),
            Err(ErrMode::Backtrack(InputError::new(
                "/01",
                ErrorKind::Verify
            )))
        );
        assert_eq!(
            date.parse_peek("2022/13/21"),
            Err(ErrMode::Backtrack(InputError::new(
                "2022/13/21",
                ErrorKind::Verify
            )))
        );
    }
}
