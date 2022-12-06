use std::ops::Deref;

use crate::Parser;
use nom::branch::alt;
use nom::bytes::complete::{take, take_until, take_while1};
use nom::character::complete::char;
use nom::combinator::cut;
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

#[cfg(test)]
mod inline_str_tests {
    use super::*;

    #[test]
    fn parens() {
        let input = "(dhsjakdhsjkadhk   dsjakldjsla  )";
        let (rest, got) = recognize_input_str(input).expect("parse str");

        assert_eq!(rest, "", "nothing remains");
        assert_eq!(got, "dhsjakdhsjkadhk   dsjakldjsla");
    }

    #[test]
    fn anyparens() {
        let input = "(dhsjakdhsjkadhk   dsjakldjsla  )";
        let (rest, got) = anyparen(input).expect("parse str");

        assert_eq!(rest, "", "nothing remains");
        assert_eq!(got, "dhsjakdhsjkadhk   dsjakldjsla");
    }
}

fn anyparen(input: &str) -> nom::IResult<&str, &str> {
    let (rest, got) = alt((
        delimited(char('('), cut(take_until(")")), take(1usize)),
        delimited(char('{'), cut(take_until("}")), take(1usize)),
        delimited(char('['), cut(take_until("]")), take(1usize)),
    ))(input)?;

    let got = got.trim_end();
    Ok((rest, got))
}

fn recognize_input_str(input: &str) -> nom::IResult<&str, &str> {
    use nom::combinator::recognize;

    let (input, tagged) = alt((
        anyparen,
        take_while1(
            |c| matches!(c, ' '| '\n' | ':' | ';' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '$' | '.' |'%' |'°' | '/' | '\\'),
        ),
        recognize(String::parse),
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
