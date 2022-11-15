
use nom::bytes::complete::take_while;
use nom::sequence::delimited;
use nom::character::complete::char;
use nom::branch::alt;

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
impl Parser for String {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, s) = alt((
            delimited(char('\''), take_while(|c| c != '\''), char('\'')),
            delimited(char('"'), take_while(|c| c != '"'), char('"')),
        ))(input)?;

        let s = s.to_string();

        Ok((input, s))
    }
}