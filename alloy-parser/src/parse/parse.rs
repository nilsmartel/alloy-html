/// Parse trait
use nom::sequence::preceded;

pub trait Parse
where
    Self: Sized,
{
    fn parse(i: &str) -> nom::IResult<&str, Self>;

    fn parse_ws(i: &str) -> nom::IResult<&str, Self> {
        preceded(crate::parse::util::whitespace, Self::parse)(i)
    }
}
