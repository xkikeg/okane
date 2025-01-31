//! Defines parser for `[syntax::PriceDBEntry]`.

use std::borrow::Cow;

use winnow::{
    ascii::{line_ending, space1},
    combinator::{seq, trace},
    stream::{AsChar, Stream, StreamIsPartial},
    ModalResult, Parser as _,
};

use crate::syntax;

use super::{
    adaptor::{ParseOptions, ParsedContext},
    character,
    error::ParseError,
    expr, primitive,
};

/// Parses a price DB content.
pub fn parse_price_db<'i>(
    options: &ParseOptions,
    input: &'i str,
) -> impl Iterator<Item = Result<(ParsedContext<'i>, syntax::PriceDBEntry<'i>), ParseError>> + 'i {
    options.parse_repeated(price_db_entry, character::newlines, input)
}

/// Parses a price DB entry line.
fn price_db_entry<'i, I>(input: &mut I) -> ModalResult<syntax::PriceDBEntry<'i>>
where
    I: Stream<Token = char, Slice = &'i str>
        + StreamIsPartial
        + winnow::stream::Compare<&'static str>
        + winnow::stream::FindSlice<(char, char)>
        + Clone,
    <I as Stream>::Token: AsChar + Clone,
{
    trace(
        "price::price_db_entry",
        seq! {syntax::PriceDBEntry {
            _: ("P", space1),
            // TODO: Support datetime, not only date.
            datetime: primitive::date.map(Into::into),
            _: space1,
            target: primitive::commodity.map(Cow::Borrowed),
            _: space1,
            rate: expr::amount,
            _: line_ending,
        }},
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use crate::parse::testing::expect_parse_ok;

    use syntax::{expr::Amount, pretty_decimal::PrettyDecimal, PriceDBEntry};

    #[test]
    fn price_db_parses_valid_with_date() {
        let input = "P 2023/12/31 JRTOK 3,584 JPY\n";

        assert_eq!(
            expect_parse_ok(price_db_entry, input),
            (
                "",
                PriceDBEntry {
                    datetime: NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2023, 12, 31).expect("2023-12-31 must exist"),
                        NaiveTime::from_hms_opt(0, 0, 0).expect("00:00:00 must exist")
                    ),
                    target: Cow::Borrowed("JRTOK"),
                    rate: Amount {
                        value: PrettyDecimal::comma3dot(dec!(3584)),
                        commodity: Cow::Borrowed("JPY")
                    },
                }
            )
        );

        let input = "P 2024-10-28 EUR 0.9367 CHF\n";

        assert_eq!(
            expect_parse_ok(price_db_entry, input),
            (
                "",
                PriceDBEntry {
                    datetime: NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2024, 10, 28).expect("2024-10-28 must exist"),
                        NaiveTime::from_hms_opt(0, 0, 0).expect("00:00:00 must exist")
                    ),
                    target: Cow::Borrowed("EUR"),
                    rate: Amount {
                        value: PrettyDecimal::unformatted(dec!(0.9367)),
                        commodity: Cow::Borrowed("CHF")
                    },
                }
            )
        );
    }

    #[ignore]
    #[test]
    fn price_db_parses_valid_with_datetime() {
        let input = "P 2022/02/02 17:06:00 DCTOPIX 22,745 JPY\n";
        assert_eq!(
            expect_parse_ok(price_db_entry, input),
            (
                "",
                PriceDBEntry {
                    datetime: NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(2022, 2, 2).expect("2022-02-02 must exist"),
                        NaiveTime::from_hms_opt(17, 6, 0).expect("17:06:00 must exist")
                    ),
                    target: Cow::Borrowed("DCTOPIX"),
                    rate: Amount {
                        value: PrettyDecimal::comma3dot(dec!(22745)),
                        commodity: Cow::Borrowed("JPY")
                    },
                }
            )
        );
    }
}
