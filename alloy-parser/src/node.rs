use nom::{
    branch::alt,
    character::complete::char,
    combinator::{cut, map, opt},
    error::context,
    multi::many0,
    sequence::{delimited, preceded, terminated},
};

use crate::{keywords::*, Body, Ident, Parser, StringInline};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub kind: Ident,
    pub ids_and_classes: Vec<IdOrClass>,
    pub attributes: Option<Attributes>,
    pub body: Body,
}
impl Parser for Node {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, kind) = Ident::parse(input)?;
        let (input, ids_and_classes) = many0(IdOrClass::parse_trim)(input)?;

        let (input, attributes) = opt(Attributes::parse_trim)(input)?;

        // may be one of these 4

        // div
        // "abcdefg"
        // ;
        // {}
        let (input, body) = cut(Body::parse_trim)(input)?;

        Ok((
            input,
            Node {
                kind,
                ids_and_classes,
                attributes,
                body,
            },
        ))
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Attributes(pub Vec<Attribute>);
impl Parser for Attributes {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        map(
            delimited(
                KeywordParenOpen::parse,
                many0(terminated(
                    Attribute::parse_trim,
                    opt(KeywordComma::parse_trim),
                )),
                KeywordParenClose::parse_trim,
            ),
            Attributes,
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdOrClass {
    Id(Ident),
    Class(Ident),
}

impl Default for IdOrClass {
    fn default() -> Self {
        IdOrClass::Id(Ident::default())
    }
}

impl Parser for IdOrClass {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        alt((
            map(
                preceded(
                    char('#'),
                    context("expect identifier after #", cut(Ident::parse)),
                ),
                IdOrClass::Id,
            ),
            map(
                preceded(
                    char('.'),
                    context("expect identifier after .", cut(Ident::parse)),
                ),
                IdOrClass::Class,
            ),
        ))(input)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub key: Ident,
    pub value: Option<String>,
}

impl Parser for Attribute {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, key) = Ident::parse(input)?;

        let Ok((input, _)) = KeywordColon::parse_trim(input) else {
            return Ok((input, Attribute { key, value: None }))
        };

        let (input, StringInline(value)) =
            context("expected attribute after :", cut(StringInline::parse_trim))(input)?;

        Ok((
            input,
            Attribute {
                key,
                value: Some(value),
            },
        ))
    }
}
