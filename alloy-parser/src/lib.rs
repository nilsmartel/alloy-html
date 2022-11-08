use nom::branch::alt;
use nom::bytes::complete::take;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while1;
use nom::character::complete::char;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;

pub fn parse(input: &str) -> nom::IResult<&str, Node> {
    let (input, node) = Node::parse_trim(input)?;
    let (input, _eolmarker) = KeywordEof::parse_trim(input)?;

    nom::combinator::not(take(1usize))(input)?;

    Ok((input, node))
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: Ident,
    pub ids_and_classes: Vec<IdOrClass>,
    pub attributes: Option<Attributes>,
    pub body: Option<Body>,
}
impl Parser for Node {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, kind) = Ident::parse(input)?;
        let (input, ids_and_classes) = many0(IdOrClass::parse_trim)(input)?;

        let (input, attributes) = opt(Attributes::parse_trim)(input)?;

        let (input, body) = opt(Body::parse_trim)(input)?;

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

#[derive(Debug, Clone)]
pub struct Body(Vec<Node>);
impl Parser for Body {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        map(
            delimited(
                char('{'),
                many0(terminated(Node::parse_trim, opt(char(',')))),
                char('}'),
            ),
            Body,
        )(input)
    }
}

#[derive(Debug, Clone)]
pub struct Attributes(Vec<Attribute>);
impl Parser for Attributes {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        map(
            delimited(
                char('('),
                many0(terminated(Attribute::parse_trim, opt(char(',')))),
                char(')'),
            ),
            Attributes,
        )(input)
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

struct KeywordEof;
impl Parser for KeywordEof {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (rest, _) = take(0usize)(input)?;
        Ok((rest, KeywordEof))
    }
}
