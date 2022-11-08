use nom::character::complete::one_of;
use nom::multi::many0;
use nom::sequence::preceded;

/// matches all whitespace at the beginning of string slice
pub fn whitespace(i: &str) -> nom::IResult<&str, &str> {
    nom::bytes::complete::take_while(|c: char| c == ' ' || c == '\n' || c == '\r' || c == '\t')(i)
}
