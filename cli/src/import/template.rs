//! Provides very minimal template mechanism for FieldPos.

use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use winnow::{
    combinator::{alt, delimited, repeat},
    error::ContextError,
    token::{one_of, take_till},
    Parser,
};

use super::config::FieldKey;

/// Template constructed from string which can render a String.
///
/// It can be instantiated with String containing `{key_name}`,
/// which will be interpolated with map from [FieldKey] to &str values.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(try_from = "&str", into = "String")]
pub struct Template {
    segments: Vec<Segment>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Segment {
    Literal(String),
    Reference(FieldKey),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid template {0}")]
    InvalidTemplate(String),
    #[error("unknown field_key {field_key}")]
    UnknownFieldKey { field_key: String },
}

impl TryFrom<&str> for Template {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<Template> for String {
    fn from(value: Template) -> Self {
        let mut ret = String::new();
        for segment in &value.segments {
            match segment {
                Segment::Literal(s) => ret.push_str(s),
                Segment::Reference(fk) => ret.push_str(&format!("{{{}}}", fk.as_str())),
            }
        }
        ret
    }
}

impl FromStr for Template {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments = repeat(
            0..,
            alt((
                delimited(
                    one_of('{'),
                    take_till(1.., b"{}").try_map(field_key_from_str),
                    one_of('}'),
                ),
                take_till(1.., b"{}")
                    .map(str::to_owned)
                    .map(Segment::Literal),
            )),
        )
        .parse(s)
        .map_err(|err: winnow::error::ParseError<&str, ContextError<_>>| {
            ParseError::InvalidTemplate(format!("{}", err))
        })?;
        Ok(Self { segments })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("field_key {0:?} in template is not supported")]
    FieldKeyUnsupported(FieldKey),
}

struct RenderedTemplate<'a> {
    parts: Vec<&'a str>,
}

impl Template {
    pub fn render<'a>(
        &'a self,
        values: &HashMap<FieldKey, &'a str>,
    ) -> Result<impl Display + 'a, RenderError> {
        let mut parts = Vec::with_capacity(self.segments.len());
        for segment in &self.segments {
            let part = match segment {
                Segment::Literal(l) => l.as_str(),
                Segment::Reference(fk) => values
                    .get(fk)
                    .ok_or(RenderError::FieldKeyUnsupported(*fk))?,
            };
            parts.push(part);
        }
        Ok(RenderedTemplate { parts })
    }
}

impl<'a> Display for RenderedTemplate<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for part in &self.parts {
            write!(f, "{}", part)?;
        }
        Ok(())
    }
}

impl FieldKey {
    fn as_str(self) -> &'static str {
        match self {
            FieldKey::Date => "date",
            FieldKey::Payee => "payee",
            FieldKey::Category => "category",
            FieldKey::Note => "note",
            FieldKey::Commodity => "commodity",
            FieldKey::SecondaryCommodity => "secondary_commodity",
            _ => "unknown",
        }
    }
}

fn field_key_from_str(s: &str) -> Result<Segment, ParseError> {
    let fk = match s {
        "date" => Ok(FieldKey::Date),
        "payee" => Ok(FieldKey::Payee),
        "category" => Ok(FieldKey::Category),
        "note" => Ok(FieldKey::Note),
        "commodity" => Ok(FieldKey::Commodity),
        "secondary_commodity" => Ok(FieldKey::SecondaryCommodity),
        _ => Err(ParseError::UnknownFieldKey {
            field_key: s.to_owned(),
        }),
    }?;
    Ok(Segment::Reference(fk))
}

#[cfg(test)]
mod tests {
    use super::*;

    use maplit::hashmap;

    #[test]
    fn render_nop() {
        let t: Template = "this is a pen".parse().unwrap();
        assert_eq!(
            "this is a pen",
            format!("{}", t.render(&HashMap::new()).unwrap())
        )
    }

    #[test]
    fn render_valid() {
        let t: Template = "{category} - {note}".parse().unwrap();
        assert_eq!(
            "Transport - SBB",
            format!(
                "{}",
                t.render(&hashmap! {
                    FieldKey::Category => "Transport",
                    FieldKey::Note => "SBB",
                })
                .unwrap()
            )
        );
    }
}
