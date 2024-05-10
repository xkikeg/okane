//! Defines parser functions for the primitive types used in Ledger format.

use crate::repl::pretty_decimal::{self, PrettyDecimal};

use chrono::NaiveDate;
use winnow::{
    branch::alt,
    bytes::{one_of, take_till1, take_while1},
    character::digit1,
    combinator::opt,
    error::{ContextError, FromExternalError, ParseError},
    IResult, Parser,
};

/// Parses comma separated decimal.
pub fn comma_decimal<'a, E>(input: &'a str) -> IResult<&str, PrettyDecimal, E>
where
    E: FromExternalError<&'a str, pretty_decimal::Error>
        + ContextError<&'a str>
        + ParseError<&'a str>,
{
    take_while1("-0123456789,.")
        .map_res(str::parse)
        .context("decimal")
        .parse_next(input)
}

/// Parses commodity in greedy manner.
/// Returns empty string if the upcoming characters are not valid as commodity to support empty commodity.
pub fn commodity<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    // Quoted commodity not supported.

    opt(take_till1(" \t\r\n0123456789.,;:?!-+*/^&|=<>[](){}@"))
        .map(|x| x.unwrap_or_default())
        .parse_next(input)
}

/// Parses date in yyyy/mm/dd format.
pub fn date<'a, E: ParseError<&'a str> + FromExternalError<&'a str, chrono::ParseError>>(
    input: &'a str,
) -> IResult<&str, NaiveDate, E> {
    alt((
        (digit1, one_of('/'), digit1, one_of('/'), digit1)
            .recognize()
            .map_res(|s| NaiveDate::parse_from_str(s, "%Y/%m/%d")),
        (digit1, one_of('-'), digit1, one_of('-'), digit1)
            .recognize()
            .map_res(|s| NaiveDate::parse_from_str(s, "%F")),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::parser::testing::expect_parse_ok;

    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

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
        let cd = comma_decimal::<winnow::error::Error<&'static str>>;
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
        let pd = date::<winnow::error::Error<&'static str>>;
        pd("not a date").unwrap_err();
        pd("2022/01").unwrap_err();
        pd("2022/13/21").unwrap_err();
    }
}
