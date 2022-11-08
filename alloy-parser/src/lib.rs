pub struct Node {
    kind: Ident,
    idsAndClasses: Vec<IdOrClass>,
    attributes: Vec<Attribute>,
    body: Vec<Node>,
}

pub enum IdOrClass {
    Id(Ident),
    Class(Ident),
}

pub struct Attribute {
    key: Ident,
    value: String,
}

pub struct Ident(String);

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
