use std::borrow::Cow;

use indexmap::IndexMap;
use nom::{
    character::{
        complete::{char, line_ending, none_of, not_line_ending, satisfy},
        is_newline,
    },
    combinator::{map, opt, recognize},
    multi::{many0_count, many1_count},
    sequence::{delimited, pair},
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
    map(
        delimited(
            char('['),
            recognize(many1_count(satisfy(|c| {
                c.is_ascii() && !c.is_control() && c != '[' && c != ']'
            }))),
            pair(char(']'), opt(line_ending)),
        ),
        |header| Cow::from(header),
    )(input)
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

    #[test]
    fn shoul_parse_group_header() {
        assert_eq!(
            Ok(("", Cow::from("header"))),
            parse_group_header("[header]")
        );
    }
}
