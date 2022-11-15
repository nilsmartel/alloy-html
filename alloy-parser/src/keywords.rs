use crate::Parser;
use nom::character::complete::char;
use nom::bytes::complete::take;

macro_rules! keyword {
    ($name: ident, $ch: expr) => {
        pub struct $name;
        impl Parser for $name {
            fn parse(input: &str) -> nom::IResult<&str, Self> {
                let (rest, _) = char($ch)(input)?;
                Ok((rest, $name))
            }
        }
    };
}

keyword!(KeywordColon, ':');
keyword!(KeywordComma, ',');
keyword!(KeywordParenOpen, '(');
keyword!(KeywordParenClose, ')');
keyword!(KeywordBracketOpen, '[');
keyword!(KeywordBracketClose, ']');
keyword!(KeywordCurlyOpen, '{');
keyword!(KeywordCurlyClose, '}');
keyword!(KeywordNone, ';');

pub struct KeywordEof;
impl Parser for KeywordEof {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (rest, _) = take(0usize)(input)?;
        Ok((rest, KeywordEof))
    }
}