use alloc::{vec, vec::Vec};
use regex::Regex;

use crate::construct_name::ConstructName;

#[derive(Default)]
pub struct Unsafe<'a> {
    pub character: char,
    pub in_construct: Option<Construct>,
    pub not_in_construct: Option<Construct>,
    pub before: Option<&'a str>,
    pub after: Option<&'a str>,
    pub at_break: bool,
    pub(crate) compiled: Option<Regex>,
}

// This could use a better name.
pub enum Construct {
    List(Vec<ConstructName>),
    Single(ConstructName),
}

impl<'a> Unsafe<'a> {
    pub fn new(
        character: char,
        before: Option<&'a str>,
        after: Option<&'a str>,
        in_construct: Option<Construct>,
        not_in_construct: Option<Construct>,
        at_break: bool,
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
                '\t',
                None,
                "[\\r\\n]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                false,
            ),
            Self::new(
                '\t',
                "[\\r\\n]".into(),
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                false,
            ),
            Self::new(
                '\t',
                None,
                None,
                Construct::List(vec![
                    ConstructName::CodeFencedLangGraveAccent,
                    ConstructName::CodeFencedLangTilde,
                ])
                .into(),
                None,
                false,
            ),
            Self::new(
                '\r',
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
                false,
            ),
            Self::new(
                '\n',
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
                false,
            ),
            Self::new(
                ' ',
                None,
                "[\\r\\n]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                false,
            ),
            Self::new(
                ' ',
                "[\\r\\n]".into(),
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                false,
            ),
            Self::new(
                ' ',
                None,
                None,
                Construct::List(vec![
                    ConstructName::CodeFencedLangGraveAccent,
                    ConstructName::CodeFencedLangTilde,
                ])
                .into(),
                None,
                false,
            ),
            Self::new(
                '!',
                None,
                "\\[".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                false,
            ),
            Self::new(
                '\"',
                None,
                None,
                Construct::Single(ConstructName::TitleQuote).into(),
                None,
                false,
            ),
            Self::new('#', None, None, None, None, true),
            Self::new(
                '#',
                None,
                "(?:[\r\n]|$)".into(),
                Construct::Single(ConstructName::HeadingAtx).into(),
                None,
                false,
            ),
            Self::new(
                '&',
                None,
                "[#A-Za-z]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                false,
            ),
            Self::new(
                '\'',
                None,
                None,
                Construct::Single(ConstructName::TitleApostrophe).into(),
                None,
                false,
            ),
            Self::new(
                '(',
                None,
                None,
                Construct::Single(ConstructName::DestinationRaw).into(),
                None,
                false,
            ),
            Self::new(
                '(',
                "\\]".into(),
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                false,
            ),
            Self::new(')', "\\d+".into(), None, None, None, true),
            Self::new(
                ')',
                None,
                None,
                Construct::Single(ConstructName::DestinationRaw).into(),
                None,
                false,
            ),
            Self::new('*', None, "(?:[ \t\r\n*])".into(), None, None, true),
            Self::new(
                '*',
                None,
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                false,
            ),
            Self::new('+', None, "(?:[ \t\r\n])".into(), None, None, true),
            Self::new('-', None, "(?:[ \t\r\n-])".into(), None, None, true),
            Self::new(
                '.',
                "\\d+".into(),
                "(?:[ \t\r\n]|$)".into(),
                None,
                None,
                true,
            ),
            Self::new('<', None, "[!/?A-Za-z]".into(), None, None, true),
            Self::new(
                '<',
                None,
                "[!/?A-Za-z]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                false,
            ),
            Self::new(
                '<',
                None,
                None,
                Construct::Single(ConstructName::DestinationLiteral).into(),
                None,
                false,
            ),
            Self::new('=', None, None, None, None, true),
            Self::new('>', None, None, None, None, true),
            Self::new(
                '>',
                None,
                None,
                Construct::Single(ConstructName::DestinationLiteral).into(),
                None,
                false,
            ),
            Self::new('[', None, None, None, None, true),
            Self::new(
                '[',
                None,
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                false,
            ),
            Self::new(
                '[',
                None,
                None,
                Construct::List(vec![ConstructName::Label, ConstructName::Reference]).into(),
                None,
                false,
            ),
            Self::new(
                '\\',
                None,
                "[\\r\\n]".into(),
                Construct::Single(ConstructName::Phrasing).into(),
                None,
                false,
            ),
            Self::new(
                ']',
                None,
                None,
                Construct::List(vec![ConstructName::Label, ConstructName::Reference]).into(),
                None,
                false,
            ),
            Self::new('_', None, None, None, None, true),
            Self::new(
                '_',
                None,
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                false,
            ),
            Self::new('`', None, None, None, None, true),
            Self::new(
                '`',
                None,
                None,
                Construct::List(vec![
                    ConstructName::CodeFencedLangGraveAccent,
                    ConstructName::CodeFencedMetaGraveAccent,
                ])
                .into(),
                None,
                false,
            ),
            Self::new(
                '`',
                None,
                None,
                Construct::Single(ConstructName::Phrasing).into(),
                Construct::List(full_phrasing_spans.clone()).into(),
                false,
            ),
            Self::new('~', None, None, None, None, true),
        ]
    }

    pub(crate) fn set_compiled(&mut self, regex_pattern: Regex) {
        self.compiled = Some(regex_pattern);
    }
}
