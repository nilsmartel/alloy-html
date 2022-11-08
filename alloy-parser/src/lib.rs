use nom::branch::alt;
use nom::bytes::complete::take;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while1;
use nom::character::complete::char;
use nom::combinator::map;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;

#[cfg(test)]
mod tests {
    use std::fs::read_dir;

    use super::*;

    #[test]
    fn all_sample_files() {
        for entry in read_dir("./samples").expect("read samples directory") {
            let entry = entry.expect("read entry");
            let name = entry.file_name();
            let name = name.to_str().unwrap();
            if !name.ends_with(".alloy") {
                continue;
            }

            let content = std::fs::read_to_string(format!("./samples/{name}"))
                .expect("read alloy samplefile");

            let res = parse(&content);
            assert!(res.is_ok(), "Parsing {name}. Result is: {res:#?}");
            let (rest, _nodetree) = res.unwrap();

            assert_eq!(
                rest, "",
                "nothing remains on file {name}. Remaining: {rest}"
            );
        }
    }

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
            assert_eq!(Ident::parse(v).unwrap().0, "", "nothing remains");
        }
    }

    #[test]
    fn inline_str() {
        let valid = [
            "'hello world'",
            "  ()()()",
            "( )",
            "max",
            "min-",
            "background-color",
            "backgroundColor",
            "background_color",
            "Ad0909-4324",
            "  ({ })",
            "  ({})[]{}",
            "  ({'hello man'})",
            "open(7)",
        ];
        for v in valid {
            let result = StringInline::parse_trim(v);
            assert!(result.is_ok(), "parsing inline str {v}");
            let (rest, _) = result.unwrap();
            assert_eq!(rest, "", "nothing remains");
        }
    }

    #[test]
    fn nodes() {
        let input = [
            "canvas#drawboard",
            "input(type: text)",
            "input(type: 'text')",
            "h1() {'hello world'}",
        ];

        for i in input {
            let result = Node::parse(i);
            assert!(result.is_ok());
            let (rest, _) = result.unwrap();

            assert_eq!(rest, "", "not rest on {i}");
        }
    }

    macro_rules! bodytest {
        ($name: ident, $input: expr) => {
            #[test]
            fn $name() {
                let input = $input;

                let result = parse(input);
                assert!(
                    result.is_ok(),
                    "expected to parse {input}. Error: {:#?}",
                    result
                );
                let (rest, _) = result.unwrap();

                assert_eq!(rest, "", "not rest on {input}");
            }
        };
    }

    bodytest!(nochildren, "head {}");
    bodytest!(attributes_on_node, "meta(charset: UTF-8)");
    bodytest!(attributes_on_node_str, "meta(charset: 'UTF-8')");

    bodytest!(embedded, "head { link(rel: stylesheet, href: xyz) }");

    bodytest!(
        body0,
        "// hello
                        html {
                            head,
                            body {}
                        }"
    );

    bodytest!(
        body1,
        "// hello
                        html {
                            head
                            body {}
                        }"
    );

    bodytest!(idsnclasses, "div#important.highlight.w-100");

    bodytest!(
        bodywithcomment,
        "html {
    head
 } /* oi ya wee wanker */ "
    );

    bodytest!(
        body2,
        "html {
    head
 }"
    );

    bodytest!(inputelem, "input(type: text)");
    bodytest!(div, "div() { h1 {} }");

    #[test]
    fn comments1() {
        let i = "// hello world\n canvas#drawboard";

        let result = parse(i);
        assert!(result.is_ok(), "expected to parse {i}");
        let (rest, _) = result.unwrap();

        assert_eq!(rest, "", "not rest on {i}");
    }

    #[test]
    fn comments2() {
        let i = "/* hello */ input(type: text) /* yeah */";

        let result = parse(i);
        assert!(
            result.is_ok(),
            "expected to parse {i}. Error: {:#?}",
            result.err()
        );
        let (rest, _) = result.unwrap();

        assert_eq!(rest, "", "not rest on {i}");
    }
}

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
pub struct Body(pub Vec<NodeOrText>);
impl Parser for Body {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, _) = KeywordCurlyOpen::parse(input)?;

        let (input, nodes) =
            many0(terminated(NodeOrText::parse_trim, opt(KeywordComma::parse_trim)))(input)?;
        let (input, _) = KeywordCurlyClose::parse_trim(input)?;

        Ok((input, Body(nodes)))
    }
}

#[derive(Debug, Clone)]
pub enum NodeOrText {
    Node(Node),
    Text(String),
}

impl Parser for NodeOrText {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        alt((
            map(Node::parse, NodeOrText::Node),
            map(StringLiteral::parse, |f| NodeOrText::Text(f.0)),
        ))(input)
    }
}

#[derive(Debug, Clone)]
pub struct Attributes(Vec<Attribute>);
impl Parser for Attributes {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        map(
            delimited(
                KeywordParenOpen::parse,
                many0(terminated(
                    Attribute::parse_trim,
                    opt(KeywordComma::parse_trim),
                )),
                KeywordParenClose::parse_trim,
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

        let (input, StringInline(value)) = StringInline::parse_trim(input)?;

        Ok((
            input,
            Attribute {
                key,
                value: Some(value),
            },
        ))
    }
}

pub struct StringInline(String);
impl Parser for StringInline {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        use nom::combinator::recognize;

        // parsing "" should ommit them
        if let Ok((rest, StringLiteral(s))) = StringLiteral::parse(input) {
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
            |c| matches!(c, ' '| '\n' | ':' | ';' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '$' | '.' ),
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
        let (input, s) = alt((
            delimited(char('\''), take_while(|c| c != '\''), char('\'')),
            delimited(char('"'), take_while(|c| c != '"'), char('"')),
        ))(input)?;

        let s = s.to_string();

        Ok((input, StringLiteral(s)))
    }
}

#[derive(Debug, Clone)]
pub struct Ident(pub String);
impl Parser for Ident {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (rest, ident) = take_while1(
            |c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '$'),
        )(input)?;

        Ok((rest, Ident(ident.to_string())))
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
}

macro_rules! keyword {
    ($name: ident, $ch: expr) => {
        struct $name;
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

struct KeywordEof;
impl Parser for KeywordEof {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (rest, _) = take(0usize)(input)?;
        Ok((rest, KeywordEof))
    }
}
