//! Defines parser functions for the primitive types used in Ledger format.

use chrono::NaiveDate;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not},
    character::complete::{char, digit1},
    combinator::{map, map_res, opt, recognize},
    error::{context, ContextError, FromExternalError, ParseError},
    sequence::tuple,
    IResult,
};
use rust_decimal::Decimal;

/// Parses number including comma, returns the decimal.
pub fn str_to_comma_decimal(x: &str) -> Result<Decimal, rust_decimal::Error> {
    x.replace(',', "").parse()
}

/// Parses comma separated decimal.
pub fn comma_decimal<'a, E>(input: &'a str) -> IResult<&str, Decimal, E>
where
    E: FromExternalError<&'a str, rust_decimal::Error>
        + ContextError<&'a str>
        + ParseError<&'a str>,
{
    context(
        "decimal",
        map_res(is_a("-0123456789,."), str_to_comma_decimal),
    )(input)
}

/// Parses commodity in greedy manner.
/// Returns empty string if the upcoming characters are not valid as commodity to support empty commodity.
pub fn commodity<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    // Quoted commodity not supported.
    map(
        opt(is_not(" \t\r\n0123456789.,;:?!-+*/^&|=<>[](){}@")),
        |x| x.unwrap_or_default(),
    )(input)
}

/// Parses date in yyyy/mm/dd format.
pub fn date<'a, E: ParseError<&'a str> + FromExternalError<&'a str, chrono::ParseError>>(
    input: &'a str,
) -> IResult<&str, NaiveDate, E> {
    alt((
        map_res(
            recognize(tuple((digit1, char('/'), digit1, char('/'), digit1))),
            |s| NaiveDate::parse_from_str(s, "%Y/%m/%d"),
        ),
        map_res(
            recognize(tuple((digit1, char('-'), digit1, char('-'), digit1))),
            |s| NaiveDate::parse_from_str(s, "%F"),
        ),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn comma_decimal_parses_valid_inputs() {
        assert_eq!(expect_parse_ok(comma_decimal, "123"), ("", dec!(123)));
        assert_eq!(
            expect_parse_ok(comma_decimal, "1,2,3.45 JPY"),
            (" JPY", dec!(123.45))
        );
        assert_eq!(
            expect_parse_ok(comma_decimal, "-012.3$"),
            ("$", dec!(-12.3))
        );
    }

    #[test]
    fn comma_decimal_fails_on_invalid_inputs() {
        let cd = comma_decimal::<nom::error::Error<&'static str>>;
        cd("不可能").unwrap_err();
        cd("!").unwrap_err();
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
        let pd = date::<nom::error::Error<&'static str>>;
        pd("not a date").unwrap_err();
        pd("2022/01").unwrap_err();
        pd("2022/13/21").unwrap_err();
    }
}
