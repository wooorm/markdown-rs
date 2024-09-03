use alloc::{vec, vec::Vec};
use regex::Regex;

use crate::ConstructName;

pub struct Unsafe<'a> {
    pub character: &'a str,
    pub in_construct: Option<Construct>,
    pub not_in_construct: Option<Construct>,
    pub before: Option<&'a str>,
    pub after: Option<&'a str>,
    pub at_break: Option<bool>,
    pub(crate) compiled: Option<Regex>,
}

// This could use a better name.
pub enum Construct {
    List(Vec<ConstructName>),
    Single(ConstructName),
}

impl<'a> Unsafe<'a> {
    fn new(
        character: &'a str,
        before: Option<&'a str>,
        after: Option<&'a str>,
        in_construct: Option<Construct>,
        not_in_construct: Option<Construct>,
        at_break: Option<bool>,
    ) -> Self {
        Unsafe {
            character,
            in_construct,
            not_in_construct,
            before,
            after,
            at_break,
            compiled: None,
        }
    }

    pub fn get_default_unsafe() -> Vec<Self> {
        let full_phrasing_spans = vec![
            ConstructName::Autolink,
            ConstructName::DestinationLiteral,
            ConstructName::DestinationRaw,
            ConstructName::Reference,
            ConstructName::TitleQuote,
            ConstructName::TitleApostrophe,
        ];

        vec![
            Self::new(
                "\t",
                None,
                r"[\\r\\n]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                None,
            ),
            Self::new(
                "\t",
                r"[\\r\\n]".into(),
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                None,
            ),
            Self::new(
                "\t",
                None,
                None,
                Construct::List(vec![
                    ConstructName::CodeFencedLangGraveAccent,
                    ConstructName::CodeFencedLangTilde,
                ])
                .into(),
                None,
                None,
            ),
            Self::new(
                "\r",
                None,
                None,
                Construct::List(vec![
                    ConstructName::CodeFencedLangGraveAccent,
                    ConstructName::CodeFencedLangTilde,
                    ConstructName::CodeFencedMetaGraveAccent,
                    ConstructName::CodeFencedMetaTilde,
                    ConstructName::DestinationLiteral,
                    ConstructName::HeadingAtx,
                ])
                .into(),
                None,
                None,
            ),
            Self::new(
                "\n",
                None,
                None,
                Construct::List(vec![
                    ConstructName::CodeFencedLangGraveAccent,
                    ConstructName::CodeFencedLangTilde,
                    ConstructName::CodeFencedMetaGraveAccent,
                    ConstructName::CodeFencedMetaTilde,
                    ConstructName::DestinationLiteral,
                    ConstructName::HeadingAtx,
                ])
                .into(),
                None,
                None,
            ),
            Self::new(
                " ",
                None,
                r"[\\r\\n]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                None,
            ),
            Self::new(
                " ",
                r"[\\r\\n]".into(),
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                None,
            ),
            Self::new(
                " ",
                None,
                None,
                Construct::List(vec![
                    ConstructName::CodeFencedLangGraveAccent,
                    ConstructName::CodeFencedLangTilde,
                ])
                .into(),
                None,
                None,
            ),
            Self::new(
                "!",
                None,
                r"\[".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                None,
            ),
            Self::new(
                "\"",
                None,
                None,
                Construct::Single(ConstructName::TitleQuote).into(),
                None,
                None,
            ),
            Self::new("#", None, None, None, None, Some(true)),
            Self::new(
                "&",
                None,
                r"[#A-Za-z]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                None,
            ),
            Self::new(
                "'",
                None,
                None,
                Construct::Single(ConstructName::TitleApostrophe).into(),
                None,
                None,
            ),
            Self::new(
                "(",
                None,
                None,
                Construct::Single(ConstructName::DestinationRaw).into(),
                None,
                None,
            ),
            Self::new(
                "(",
                r"\]".into(),
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                None,
            ),
            Self::new(")", r"\d+".into(), None, None, None, Some(true)),
            Self::new(
                ")",
                None,
                None,
                Construct::Single(ConstructName::DestinationRaw).into(),
                None,
                None,
            ),
            Self::new("*", None, r"(?:[ \t\r\n*])".into(), None, None, Some(true)),
            Self::new(
                "*",
                None,
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                None,
            ),
            Self::new("+", None, r"(?:[ \t\r\n])".into(), None, None, Some(true)),
            Self::new("-", None, r"(?:[ \t\r\n-])".into(), None, None, Some(true)),
            Self::new(
                ".",
                r"\d+".into(),
                "(?:[ \t\r\n]|$)".into(),
                None,
                None,
                Some(true),
            ),
            Self::new("<", None, r"[!/?A-Za-z]".into(), None, None, Some(true)),
            Self::new(
                "<",
                None,
                "[!/?A-Za-z]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                None,
            ),
            Self::new(
                "<",
                None,
                None,
                Construct::Single(ConstructName::DestinationLiteral).into(),
                None,
                None,
            ),
            Self::new("=", None, None, None, None, Some(true)),
            Self::new(">", None, None, None, None, Some(true)),
            Self::new(
                ">",
                None,
                None,
                Construct::Single(ConstructName::DestinationLiteral).into(),
                None,
                Some(true),
            ),
            Self::new("[", None, None, None, None, Some(true)),
            Self::new(
                "[",
                None,
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                None,
            ),
            Self::new(
                "[",
                None,
                None,
                Construct::List(vec![ConstructName::Label, ConstructName::Reference]).into(),
                None,
                None,
            ),
            Self::new(
                r"\",
                None,
                "[\\r\\n]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                None,
            ),
            Self::new(
                "]",
                None,
                None,
                Construct::List(vec![ConstructName::Label, ConstructName::Reference]).into(),
                None,
                None,
            ),
            Self::new("_", None, None, None, None, Some(true)),
            Self::new(
                "_",
                None,
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                None,
            ),
            Self::new("`", None, None, None, None, Some(true)),
            Self::new(
                "`",
                None,
                None,
                Construct::List(vec![
                    ConstructName::CodeFencedLangGraveAccent,
                    ConstructName::CodeFencedMetaGraveAccent,
                ])
                .into(),
                None,
                None,
            ),
            Self::new(
                "`",
                None,
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                None,
            ),
            Self::new("~", None, None, None, None, Some(true)),
        ]
    }

    pub(crate) fn is_compiled(&self) -> bool {
        self.compiled.is_some()
    }

    pub(crate) fn set_compiled(&mut self, regex_pattern: Regex) {
        self.compiled = Some(regex_pattern);
    }
}
