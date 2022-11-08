use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while1;
use nom::character::complete::char;
use nom::combinator::map;
use nom::combinator::recognize;
use nom::sequence::delimited;
use nom::sequence::preceded;

pub struct Node {
    pub kind: Ident,
    pub ids_and_classes: Vec<IdOrClass>,
    pub attributes: Vec<Attribute>,
    pub body: Vec<Node>,
}
impl Parser for Node {}

pub enum IdOrClass {
    Id(Ident),
    Class(Ident),
}

impl Parser for IdOrClass {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        alt((
            map(preceded(char('#'), Ident::parse), IdOrClass::Id),
            map(preceded(char('.'), Ident::parse), IdOrClass::Class),
        ))(input)
    }
}

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

        let (input, value) = recognize(recognize_input_str)(input)?;

        let value = value.trim();
        let value = Some(value.to_string());

        Ok((input, Attribute { key, value }))
    }
}

fn recognize_input_str(input: &str) -> nom::IResult<&str, &str> {
    fn anyparen(input: &str) -> nom::IResult<&str, &str> {
        alt((
            delimited(char('('), recognize_input_str, char(')')),
            delimited(char('{'), recognize_input_str, char('}')),
            delimited(char('['), recognize_input_str, char(']')),
        ))(input)
    }

    let (input, tagged) = alt((
        take_while1(
            |c| matches!(c, ' ' | ':' | ';' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '$' ),
        ),
        recognize(StringLiteral::parse),
        anyparen,
    ))(input)?;

    if let Ok((input, tagged)) = recognize_input_str(input) {
        return Ok((input, tagged));
    }

    Ok((input, tagged))
}

struct StringLiteral(String);
impl Parser for StringLiteral {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, _) = char('\'')(input)?;

        let (input, s) = take_while(|c| c == '\'')(input)?;
        let s = s.to_string();

        let (input, _) = char('\'')(input)?;

        Ok((input, StringLiteral(s)))
    }
}

pub struct Ident(pub String);
impl Parser for Ident {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (rest, ident) = take_while1(
            |c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '$' ),
        )(input)?;

        Ok((rest, Ident(ident.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers() {
        let valid = [
            "max",
            "min-",
            "background-color",
            "backgroundColor",
            "background_color",
            "Ad0909-4324",
        ];
        for v in valid {
            assert!(Ident::parse(v).is_ok());
        }
    }
}

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
                return Self::parse_trim(&input[index..]);
            }
            return Self::parse_trim("");
        }
        /* cut out comments */
        if input.starts_with("/*") {
            if let Some(index) = input.find("*/") {
                return Self::parse_trim(&input[index..]);
            }
            // It's allowed to simply cut off all remaining content without closing */
            return Self::parse_trim("");
        }

        Self::parse(input)
    }
}

struct KeywordColon;
impl Parser for KeywordColon {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (rest, _) = char(':')(input)?;
        Ok((rest, KeywordColon))
    }
}
