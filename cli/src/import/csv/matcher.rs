use regex::Regex;

use crate::import::{config, extract, ImportError};

/// Record is a wrapper to represent a CSV line.
#[derive(Debug, Copy, Clone)]
pub struct Record<'a> {
    pub payee: &'a str,
    pub category: Option<&'a str>,
    pub secondary_commodity: Option<&'a str>,
}

#[derive(Debug)]
enum MatchField {
    Payee,
    Category,
    SecondaryCommodity,
}

/// Matcher for CSV.
#[derive(Debug)]
pub struct CsvMatcher {
    field: MatchField,
    pattern: Regex,
}

impl<'a> extract::Entity<'a> for CsvMatcher {
    type T = Record<'a>;
}

impl TryFrom<(config::RewriteField, &str)> for CsvMatcher {
    type Error = ImportError;

    fn try_from((field, pattern): (config::RewriteField, &str)) -> Result<CsvMatcher, Self::Error> {
        let field = match field {
            config::RewriteField::Payee => Ok(MatchField::Payee),
            config::RewriteField::Category => Ok(MatchField::Category),
            config::RewriteField::SecondaryCommodity => Ok(MatchField::SecondaryCommodity),
            _ => Err(ImportError::InvalidConfig(
                "CSV only supports payee, category or secondary_commodity matcher.",
            )),
        }?;
        let pattern = extract::regex_matcher(pattern)?;
        Ok(CsvMatcher { field, pattern })
    }
}

impl extract::EntityMatcher for CsvMatcher {
    fn captures<'a>(
        &self,
        fragment: &extract::Fragment<'a>,
        entity: Record<'a>,
    ) -> Option<extract::Matched<'a>> {
        match self.field {
            MatchField::Payee => {
                let payee = fragment.payee.unwrap_or(entity.payee);
                self.pattern.captures(payee).map(|c| c.into())
            }
            MatchField::Category => entity
                .category
                .and_then(|c| self.pattern.captures(c))
                .and(Some(extract::Matched::default())),
            MatchField::SecondaryCommodity => entity
                .secondary_commodity
                .and_then(|c| self.pattern.captures(c))
                .and(Some(extract::Matched::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use extract::EntityMatcher as _;

    #[test]
    fn matcher_fails_with_unsupported_field() {
        CsvMatcher::try_from((config::RewriteField::DomainCode, "foo"))
            .expect_err("domain_code is not a supported field");
    }

    #[test]
    fn matches_secondary_commodity() {
        let m: CsvMatcher = ((config::RewriteField::SecondaryCommodity, "C.F"))
            .try_into()
            .unwrap();

        let got = m.captures(
            &extract::Fragment::default(),
            Record {
                payee: "this is the shop",
                category: None,
                secondary_commodity: Some("CHF"),
            },
        );
        assert_eq!(
            Some(extract::Matched {
                payee: None,
                code: None
            }),
            got
        );
    }
}
