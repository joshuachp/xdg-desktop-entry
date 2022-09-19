use std::borrow::Cow;

use indexmap::IndexMap;
use nom::{
    character::complete::{char, line_ending, not_line_ending},
    combinator::{map, recognize},
    multi::many0_count,
    sequence::pair,
    IResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Line<'a> {
    Comment(Cow<'a, str>),
    EmptyLine,
    Group {
        header: Cow<'a, str>,
        entries: IndexMap<Key, Value>,
    },
}

type Key = String;
type Value = String;

fn parse_comment(input: &str) -> IResult<&str, Line> {
    map(recognize(pair(char('#'), not_line_ending)), |comment| {
        Line::Comment(Cow::from(comment))
    })(input)
}

fn parse_empty_line(input: &str) -> IResult<&str, Line> {
    map(line_ending, |_| Line::EmptyLine)(input)
}

fn parse_group(input: &str) -> IResult<&str, Line> {
    todo!()
    // recognize(pair(, second))
}

fn parse_group_header(input: &str) -> IResult<&str, Cow<str>> {
    todo!();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn shoul_parse_comment() {
        assert_eq!(
            Ok(("\n", Line::Comment(Cow::from("# Code")))),
            parse_comment("# Code\n")
        )
    }

    #[test]
    fn shoul_parse_empty_comment() {
        assert_eq!(Ok(("", Line::Comment(Cow::from("#")))), parse_comment("#"))
    }

    #[test]
    fn shoul_parse_empty_line() {
        assert_eq!(Ok(("", Line::EmptyLine)), parse_empty_line("\n"))
    }
}
