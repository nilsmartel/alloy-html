use alloy_parser as ast;
use std::{
    ffi::OsString,
    io::{self, stdout, BufWriter, Write},
};

use structopt::StructOpt;

#[derive(StructOpt)]
/// CLI to transform Alloy files into html
struct Config {
    #[structopt()]
    infile: OsString,
}

fn main() {
    let Config { infile } = Config::from_args();

    let content = std::fs::read_to_string(infile).expect("read input file");

    let node = ast::parse(&content).expect("parse valid input").1;

    let out = stdout();
    let out = out.lock();
    let mut out = BufWriter::new(out);

    let config = OutputConfig { indent: 2 };

    to_html(&mut out, &node, 0, &config).expect("write to stdout");

    out.flush().expect("flush to stdout");
}

#[derive(Debug)]
struct OutputConfig {
    indent: usize,
}

fn to_html(
    w: &mut impl Write,
    nodes: &[ast::Element],
    level: usize,
    config: &OutputConfig,
) -> io::Result<()> {
    for node in nodes {
        to_html_node(w, node, level, config)?;
    }
    Ok(())
}

fn to_html_node(
    w: &mut impl Write,
    node: &ast::Element,
    level: usize,
    config: &OutputConfig,
) -> io::Result<()> {
    // proper indentation
    indent(w, level * config.indent)?;

    let node = match node {
        ast::Element::Text(t) => {
            let t = escape(&t);
            writeln!(w, "{t}")?;
            return Ok(());
        }
        ast::Element::Node(node) => node,
    };

    // write start of html tag.
    write!(w, "<{}", node.kind.0)?;

    let mut classes = Vec::new();
    for ioc in node.ids_and_classes.iter() {
        match ioc {
            ast::IdOrClass::Id(i) => {
                write!(w, " id='{}'", i.0)?;
            }
            ast::IdOrClass::Class(c) => {
                classes.push(c.0.clone());
            }
        }
    }

    if !classes.is_empty() {
        let classes = classes.join(" ");

        write!(w, " class='{classes}'")?;
    }

    if let Some(ref attrs) = node.attributes {
        for attr in attrs.0.iter() {
            write!(w, " {}", attr.key.0)?;
            let Some(ref value) = attr.value else {
                continue;
            };
            write!(w, "='{}'", escape(value))?;
        }
    }

    writeln!(w, ">")?;

    to_html(w, &node.body, level + 1, config)?;

    indent(w, level * config.indent)?;
    writeln!(w, "</{}>", node.kind.0)?;

    Ok(())
}

fn indent(w: &mut impl Write, times: usize) -> io::Result<()> {
    for _ in 0..times {
        write!(w, " ")?;
    }

    Ok(())
}

fn escape(i: &str) -> String {
    let mut s = String::new();
    for c in i.chars() {
        match c {
            '\'' => {
                s.push_str("\\'");
            }
            // '\n' => {
            //     s.push_str("<br />");
            // }
            x => {
                s.push(x);
            }
        };
    }

    s
}
