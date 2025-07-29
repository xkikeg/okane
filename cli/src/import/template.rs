//! Provides very minimal template mechanism for FieldPos.

use std::{
    fmt::{Debug, Display},
    num::ParseIntError,
    str::FromStr,
};

use one_based::OneBasedU32;
use serde::{Deserialize, Serialize};
use winnow::{
    combinator::{alt, delimited, repeat},
    error::ContextError,
    token::{one_of, take_till},
    Parser,
};

use super::config::FieldKey;

/// Key used in the template.
/// It could be either `{<field_key_name>}`,
/// or `{<number>}`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TemplateKey {
    /// Named field.
    Named(FieldKey),
    /// Indexed field, specified as 0-origin index.
    Indexed(OneBasedU32),
}

impl Display for TemplateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // 1-origin index for UI.
            TemplateKey::Indexed(i) => write!(f, "{}", i),
            TemplateKey::Named(fk) => write!(f, "{}", fk.as_str()),
        }
    }
}

impl PartialEq<FieldKey> for TemplateKey {
    fn eq(&self, other: &FieldKey) -> bool {
        match self {
            TemplateKey::Named(fk) => fk == other,
            TemplateKey::Indexed(_) => false,
        }
    }
}

/// Template constructed from string which can render a String.
///
/// It can be instantiated with String containing `{key_name}`,
/// which will be interpolated with map from [FieldKey] to &str values.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Template {
    segments: Vec<Segment>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid template: {0}")]
    InvalidTemplate(String),
    #[error("unknown template_key {template_key}: template key must be a positive integer or known field key")]
    UnknownTemplateKey { template_key: String },
    #[error("index-based template key must be a positive integer")]
    InvalidIndexTemplateKey(#[from] ParseIntError),
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Segment {
    Literal(String),
    Reference(TemplateKey),
}

impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Literal(s) => write!(f, "{}", s),
            Segment::Reference(tk) => write!(f, "{{{}}}", tk),
        }
    }
}

impl TryFrom<&str> for Template {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in &self.segments {
            write!(f, "{}", segment)?;
        }
        Ok(())
    }
}

impl FromStr for Template {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments = repeat(
            0..,
            alt((
                delimited(
                    one_of::<_, _, ContextError>('{'),
                    take_till(1.., b"{}").try_map(template_key_from_str),
                    one_of('}'),
                )
                .map(Segment::Reference),
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

impl Serialize for Template {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Template {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let s: &str = Deserialize::deserialize(deserializer)?;
        s.parse()
            .map_err(|err| D::Error::custom(format!("failed to parse the template: {}", err)))
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum RenderError {
    #[error("template field {0} in template is not supported")]
    FieldKeyUnsupported(TemplateKey),
    #[error("template field {0:?} is self recurring in the template")]
    SelfRecursiveFieldKey(FieldKey),
}

#[derive(Debug)]
struct RenderedTemplate<'a> {
    parts: Vec<&'a str>,
}

/// Trait feeding the value of template interpolated variables to the [`Template`].
pub trait Interpolate<'a> {
    /// Returns the `&str` corresponding to the given `key`.
    /// It may return `None` if it's impossible.
    fn interpolate(&self, key: TemplateKey) -> Option<&'a str>;
}

impl Template {
    /// Renders the template.
    /// Args:
    ///  * `current_field`: Currently rendered field to avoid infinite recursion.
    pub fn render<'a, T: Interpolate<'a>>(
        &'a self,
        current_field: FieldKey,
        values: T,
    ) -> Result<impl Debug + Display + 'a, RenderError> {
        let mut parts = Vec::with_capacity(self.segments.len());
        for segment in &self.segments {
            let part = match segment {
                Segment::Literal(l) => l.as_str(),
                Segment::Reference(fk) if *fk == current_field => {
                    Err(RenderError::SelfRecursiveFieldKey(current_field))?
                }
                Segment::Reference(fk) => values
                    .interpolate(*fk)
                    .ok_or(RenderError::FieldKeyUnsupported(*fk))?,
            };
            parts.push(part);
        }
        Ok(RenderedTemplate { parts })
    }
}

impl Display for RenderedTemplate<'_> {
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

fn template_key_from_str(s: &str) -> Result<TemplateKey, ParseError> {
    if s.chars().all(|x| x.is_ascii_digit()) {
        let v: OneBasedU32 = s.parse()?;
        return Ok(TemplateKey::Indexed(v));
    }
    match s {
        "date" => Ok(TemplateKey::Named(FieldKey::Date)),
        "payee" => Ok(TemplateKey::Named(FieldKey::Payee)),
        "category" => Ok(TemplateKey::Named(FieldKey::Category)),
        "note" => Ok(TemplateKey::Named(FieldKey::Note)),
        "commodity" => Ok(TemplateKey::Named(FieldKey::Commodity)),
        "secondary_commodity" => Ok(TemplateKey::Named(FieldKey::SecondaryCommodity)),
        _ => Err(ParseError::UnknownTemplateKey {
            template_key: s.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use one_based::OneBasedUsize;

    use crate::one_based_macro::one_based_32;

    /// impl for RenderValue with simple Vec.
    impl<'a> Interpolate<'a> for Vec<(FieldKey, &'a str)> {
        fn interpolate(&self, key: TemplateKey) -> Option<&'a str> {
            match key {
                TemplateKey::Named(fk) => self
                    .iter()
                    .filter_map(|(key, value)| if *key == fk { Some(*value) } else { None })
                    .next(),
                TemplateKey::Indexed(i) => self
                    .get(OneBasedUsize::try_from(i).unwrap().as_zero_based())
                    .map(|(_, v)| *v),
            }
        }
    }

    #[test]
    fn wrong_template() {
        // no support for {} escaping yet. this must be improved though.
        assert!(matches!(
            Template::from_str("{{}}").unwrap_err(),
            ParseError::InvalidTemplate(_)
        ));

        // invalid number
        assert!(matches!(
            Template::from_str("{-123}").unwrap_err(),
            ParseError::InvalidTemplate(_)
        ));

        // unknown FieldKey
        assert!(matches!(
            Template::from_str("{unknown}").unwrap_err(),
            ParseError::InvalidTemplate(_)
        ));
    }

    #[test]
    fn render_plain_str() {
        let t: Template = "this is a pen".parse().unwrap();
        assert_eq!(
            "this is a pen",
            t.render(FieldKey::Payee, Vec::new()).unwrap().to_string()
        );
    }

    #[test]
    fn render_valid_index() {
        let t: Template = "{1} - {2}".parse().unwrap();
        assert_eq!(
            "Transport - SBB",
            t.render(
                FieldKey::Payee,
                vec![(FieldKey::Category, "Transport"), (FieldKey::Note, "SBB"),]
            )
            .unwrap()
            .to_string()
        );
    }

    #[test]
    fn render_index_out_of_range() {
        let t: Template = "{3}".parse().unwrap();
        assert_eq!(
            RenderError::FieldKeyUnsupported(TemplateKey::Indexed(one_based_32!(3))),
            t.render(
                FieldKey::Payee,
                vec![(FieldKey::Category, "Transport"), (FieldKey::Note, "SBB"),]
            )
            .unwrap_err(),
        );
    }

    #[test]
    fn render_valid_name() {
        let t: Template = "{category} - {note}".parse().unwrap();
        assert_eq!(
            "Transport - SBB",
            t.render(
                FieldKey::Payee,
                vec![(FieldKey::Category, "Transport"), (FieldKey::Note, "SBB"),]
            )
            .unwrap()
            .to_string()
        );
    }

    #[test]
    fn render_unknown_field() {
        let t: Template = "{commodity} - {secondary_commodity}".parse().unwrap();
        assert_eq!(
            RenderError::FieldKeyUnsupported(TemplateKey::Named(FieldKey::Commodity)),
            t.render(
                FieldKey::Payee,
                vec![(FieldKey::Category, "Transport"), (FieldKey::Note, "SBB"),]
            )
            .unwrap_err(),
        );
    }

    #[test]
    fn render_self_recursive() {
        let t: Template = "{category} - {date}".parse().unwrap();
        assert_eq!(
            RenderError::SelfRecursiveFieldKey(FieldKey::Category),
            t.render(
                FieldKey::Category,
                vec![(FieldKey::Category, "Transport"), (FieldKey::Note, "SBB"),]
            )
            .unwrap_err(),
        );
    }
}
