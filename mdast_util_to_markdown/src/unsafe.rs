use alloc::{vec, vec::Vec};
use regex::Regex;

use crate::construct_name::ConstructName;

#[derive(Default)]
pub struct Unsafe<'a> {
    // TODO this could be a char
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
    pub fn new(
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
                "[\\r\\n]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                None,
            ),
            Self::new(
                "\t",
                "[\\r\\n]".into(),
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
                "[\\r\\n]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                None,
            ),
            Self::new(
                " ",
                "[\\r\\n]".into(),
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
                "\\[".into(),
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
                "#",
                None,
                "(?:[\r\n]|$)".into(),
                Construct::Single(ConstructName::HeadingAtx).into(),
                None,
                None,
            ),
            Self::new(
                "&",
                None,
                "[#A-Za-z]".into(),
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
                "\\]".into(),
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                None,
            ),
            Self::new(")", "\\d+".into(), None, None, None, Some(true)),
            Self::new(
                ")",
                None,
                None,
                Construct::Single(ConstructName::DestinationRaw).into(),
                None,
                None,
            ),
            Self::new("*", None, "(?:[ \t\r\n*])".into(), None, None, Some(true)),
            Self::new(
                "*",
                None,
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                None,
            ),
            Self::new("+", None, "(?:[ \t\r\n])".into(), None, None, Some(true)),
            Self::new("-", None, "(?:[ \t\r\n-])".into(), None, None, Some(true)),
            Self::new(
                ".",
                "\\d+".into(),
                "(?:[ \t\r\n]|$)".into(),
                None,
                None,
                Some(true),
            ),
            Self::new("<", None, "[!/?A-Za-z]".into(), None, None, Some(true)),
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
                "\\",
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

    pub(crate) fn set_compiled(&mut self, regex_pattern: Regex) {
        self.compiled = Some(regex_pattern);
    }
}
