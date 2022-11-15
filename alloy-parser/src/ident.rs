
use nom::bytes::complete::take_while1;

use super::Parser;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Ident(pub String);

impl Parser for Ident {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (rest, ident) = take_while1(
            |c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '$' | '%' | '°'),
        )(input)?;

        Ok((rest, Ident(ident.to_string())))
    }
}
