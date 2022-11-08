fn main() {
    let infile = std::env::args().nth(1).expect("input file as argument");
    let content = std::fs::read_to_string(infile).expect("read input file");

    let result = alloy_parser::parse(&content);

    println!("{result:#?}");
}
