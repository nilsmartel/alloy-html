use parse::Parse;
mod parse;
mod util;

pub struct Ident(String);
impl Parse for Ident {
    fn parse(i: &str) -> nom::IResult<&str, Self> {
        use nom::bytes::complete::take_while;
        use nom::character::complete::alpha1;
        use nom::character::is_alphanumeric;
        use nom::combinator::map;
        use nom::sequence::pair;

        map(
            pair(
                alpha1,
                take_while(|c: char| is_alphanumeric(c as u8) || c == '_' || c == '-'),
            ),
            |(a, b)| Ident(format!("{}{}", a, b)),
        )(i)
    }
}

pub struct Id(Ident);
impl Parse for Id {
    fn parse(i: &str) -> nom::IResult<&str, Self> {
        use nom::character::complete::char;
        use nom::combinator::map;
        use nom::sequence::preceded;

        map(preceded(char('#'), Ident::parse_ws), Id)(i)
    }
}

pub struct Class(Ident);
impl Parse for Class {
    fn parse(i: &str) -> nom::IResult<&str, Self> {
        use nom::character::complete::char;
        use nom::combinator::map;
        use nom::sequence::preceded;

        map(preceded(char('.'), Ident::parse_ws), Class)(i)
    }
}

/// Specifies an HTML Element
pub struct Element {
    kind: Ident,
    id: Option<Id>,
    classes: Vec<Class>,
    // TODO attributes
}

impl Parse for Element {
    fn parse(i: &str) -> nom::IResult<&str, Self> {
        use nom::{combinator::opt, multi::many0};
        let (i, kind) = Ident::parse(i)?;
        let (i, id) = opt(Id::parse_ws)(i)?;
        let (i, classes) = many0(Class::parse_ws)(i)?;

        Ok((i, Element { kind, id, classes }))
    }
}
