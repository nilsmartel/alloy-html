use nom::branch::alt;
use nom::bytes::complete::take;
use nom::bytes::complete::take_while1;
use nom::character::complete::char;
use nom::sequence::delimited;

mod ident;
pub use ident::Ident;
mod parser;
pub use parser::*;
mod body;
pub use body::*;
mod keywords;
use keywords::*;

mod node;
pub use node::*;

mod element;
pub use element::Element;

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
    fn quick_syntax() {
        let input = "div p 'hello'";
        let expected = Node {
            kind: Ident(String::from("div")),
            body: Body::from_s("p {'hello'}"),
            ..Default::default()
        };

        let result = Node::from_s(input);

        assert_eq!(result, expected);
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
            "canvas#drawboard;",
            "input(type: text);",
            "input(type: 'text');",
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

    #[test]
    fn fullelement() {
        let input = "img(src: ../resources/icon.png, onclick: goto('home'));";
        let expected = Node {
            kind: Ident("img".to_string()),
            ids_and_classes: Vec::new(),
            attributes: Some(Attributes(vec![
                Attribute {
                    key: Ident("src".to_string()),
                    value: Some("../resources/icon.png".to_string()),
                },
                Attribute {
                    key: Ident("onclick".to_string()),
                    value: Some("goto('home')".to_string()),
                },
            ])),
            body: Body::default(),
        };

        let Ok((rest, mut result)) = parse(input) else {
                panic!("expected to parse input");
            };
        let Element::Node(result) = result.remove(0) else {
            panic!("expected node");
        };

        assert_eq!(rest, "");
        assert_eq!(expected.kind, result.kind, "same kind");
        assert_eq!(
            expected.ids_and_classes, result.ids_and_classes,
            "same ids and classes"
        );
        assert_eq!(expected.attributes, result.attributes, "same attributes");
        assert_eq!(expected.body, result.body, "same body");
    }

    bodytest!(nochildren, "head {}");
    bodytest!(attributes_on_node, "meta(charset: UTF-8);");
    bodytest!(attributes_on_node_str, "meta(charset: 'UTF-8');");

    bodytest!(embedded, "head { link(rel: stylesheet, href: xyz); }");
    bodytest!(
        embedded1,
        "head { link(rel: stylesheet, href: xyz); style \"\" }"
    );

    bodytest!(stylesheet1, "style {{}}");

    bodytest!(textnode1, "h1 'hello world'");
    bodytest!(textnode2, "h1(bold: true) 'hello world'");
    bodytest!(textnode3, "h1.bold 'hello world'");

    bodytest!(
        head1,
        "
    head {
        /*
        meta(charset: UTF-8);
        link(
            rel: stylesheet,
            href: \"https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css\"
        );

        style {{
            .h-100 {
                height: 100%
            }
        }}
        */
    }
    "
    );

    bodytest!(
        head2,
        "
    head {
        meta(charset: UTF-8);
        link(
            rel: stylesheet,
            href: \"https://stackpath.bootstrapcdn.com/bootstrap/4.3.1/css/bootstrap.min.css\"
        );

        style '
            .h-100 {
                height: 100%
            }
        '
    }"
    );

    bodytest!(body_filled, "
    body {
        div#header.w-100(style: 'height: 48px; margin-top: 8px') {
                                                    //   ________ <- Note how the opening and closing parens are still getting counted
            img(src: ../ressources/icon.png, onclick: goto('home'));

            h2.color-green { Graphmasters }
            input(type: 'text');
        }
    }
    ");

    #[test]
    fn attributes_and_ids() {
        let input = "div#header.w-100(style: 'height: 48px; margin-top: 8px') {
        }";
        let expected = Node {
            kind: Ident::from_s("div"),
            ids_and_classes: vec![IdOrClass::from_s("#header"), IdOrClass::from_s(".w-100")],
            attributes: Some(Attributes(vec![Attribute {
                key: Ident::from_s("style"),
                value: Some(String::from("height: 48px; margin-top: 8px")),
            }])),
            body: Body::default(),
        };

        let result = Node::from_s(input);

        assert_eq!(result, expected);
    }

    bodytest!(div_filled, "
        div#header.w-100(style: 'height: 48px; margin-top: 8px') {
                                                    //   ________ <- Note how the opening and closing parens are still getting counted
            img(src: ../ressources/icon.png, onclick: goto('home'));

            h2.color-green { 'Graphmasters' }
            input(type: 'text');
        }
    ");

    bodytest!(
        div_empty,
        "
        div#header.w-100(style: 'height: 48px; margin-top: 8px') {}
    "
    );

    bodytest!(
        div_empty2,
        "
        div#header.w-100(style: 'height: 48px; margin-top: 8px');
    "
    );

    bodytest!(
        image,
        "
            img(src: ../ressources/icon.png, onclick: goto('home'));
    "
    );

    bodytest!(
        body0,
        "// hello
                        html {
                            head;,
                            body {}
                        }"
    );

    bodytest!(
        body1,
        "// hello
                        html {
                            head;
                            body {}
                        }"
    );

    bodytest!(idsnclasses, "div#important.highlight.w-100;");

    bodytest!(
        bodywithcomment,
        "html {
    head;
 } /* oi ya wee wanker */ "
    );

    bodytest!(
        body2,
        "html {
    head;
 }"
    );

    bodytest!(inputelem, "input(type: text);");
    bodytest!(div, "div() { h1 {} }");

    #[test]
    fn comments1() {
        let i = "// hello world\n canvas#drawboard;";

        let result = parse(i);
        assert!(result.is_ok(), "expected to parse {i}");
        let (rest, _) = result.unwrap();

        assert_eq!(rest, "", "not rest on {i}");
    }

    #[test]
    fn inline_str2() {
        let i = "../../src/main.rs";

        let result = StringInline::parse(i);
        assert!(result.is_ok(), "expected to parse {i}");
        let (rest, r) = result.unwrap();
        assert_eq!(rest, "", "not rest on {i}");
        assert_eq!(r.0, i);
    }

    #[test]
    fn comments2() {
        let i = "/* hello */ input(type: text); /* yeah */";

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

pub fn parse(input: &str) -> nom::IResult<&str, Body> {
    let (input, body) = Body::parse_trim(input)?;

    let (input, _eolmarker) = KeywordEof::parse_trim(input)?;

    nom::combinator::not(take(1usize))(input)?;

    Ok((input, body))
}

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
            |c| matches!(c, ' '| '\n' | ':' | ';' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '$' | '.' |'%' |'°' | '/' | '\\'),
        ),
        recognize(String::parse),
        anyparen,
    ))(input)?;

    if let Ok((input, tagged)) = recognize_input_str(input) {
        return Ok((input, tagged));
    }

    Ok((input, tagged))
}
