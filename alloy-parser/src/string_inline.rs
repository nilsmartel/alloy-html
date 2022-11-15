use std::ops::Deref;

use crate::Parser;
use nom::branch::alt;
use nom::bytes::complete::{take, take_while1};
use nom::character::complete::char;
use nom::sequence::delimited;

/// Represents a special syntax by which we can recognize strings inside attributes.
/// Designed to be most compatible with javascript and respects opening / closing brackets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringInline(pub String);
impl Parser for StringInline {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        use nom::combinator::recognize;

        // parsing "" should ommit them
        if let Ok((rest, s)) = String::parse(input) {
            return Ok((rest, StringInline(s)));
        }

        let (rest, s) = recognize(recognize_input_str)(input)?;
        Ok((rest, StringInline(s.to_string())))
    }
}

fn recognize_input_str(input: &str) -> nom::IResult<&str, &str> {
    use nom::combinator::recognize;

    fn anyparen(input: &str) -> nom::IResult<&str, &str> {
        alt((
            delimited(
                char('('),
                alt((recognize_input_str, take(0usize))),
                char(')'),
            ),
            delimited(
                char('{'),
                alt((recognize_input_str, take(0usize))),
                char('}'),
            ),
            delimited(
                char('['),
                alt((recognize_input_str, take(0usize))),
                char(']'),
            ),
        ))(input)
    }

    let (input, tagged) = alt((
        take_while1(
            |c| matches!(c, ' '| '\n' | ':' | ';' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '$' | '.' |'%' |'°' | '/' | '\\'),
        ),
        recognize(String::parse),
        anyparen,
    ))(input)?;

    if let Ok((input, tagged)) = recognize_input_str(input) {
        return Ok((input, tagged));
    }

    Ok((input, tagged))
}

impl Deref for StringInline {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
