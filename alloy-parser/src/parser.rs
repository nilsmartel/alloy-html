use nom::branch::alt;
use nom::bytes::complete::{take_while, take_until, take};
use nom::character::complete::char;
use nom::combinator::{map, cut};
use nom::sequence::{delimited, preceded};

use crate::keywords::KeywordInline;
use crate::StringInline;

pub trait Parser
where
    Self: Sized,
{
    fn parse(input: &str) -> nom::IResult<&str, Self>;

    fn parse_trim(input: &str) -> nom::IResult<&str, Self> {
        let input = input.trim_start();

        // cut out commments
        if input.starts_with("//") {
            if let Some(index) = input.find('\n') {
                let index = index + 1;
                return Self::parse_trim(&input[index..]);
            }
            return Self::parse_trim("");
        }
        /* cut out comments */
        if input.starts_with("/*") {
            if let Some(index) = input.find("*/") {
                let index = index + 2;
                return Self::parse_trim(&input[index..]);
            }
            // It's allowed to simply cut off all remaining content without closing */
            return Self::parse_trim("");
        }

        Self::parse(input)
    }

    fn from_s(s: &str) -> Self {
        Self::parse_trim(s).unwrap().1
    }
}

/// "hello world"
/// 'hello world'
/// `hello world`
/// ${ ... }
impl Parser for String {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        alt((
            map(
                delimited(char('\''), cut(take_until("'")), take(1usize)),
                String::from,
            ),
            map(
                delimited(char('\"'), cut(take_until("\"")), take(1usize)),
                String::from,
            ),
            map(
                delimited(char('`'), cut(take_until("`")), take(1usize)),
                String::from,
            ),
            // TODO is this needed for css? try in production or remove
            preceded(KeywordInline::parse, map(StringInline::parse, |s| s.0)),
        ))(input)
    }
}
