use std::borrow::Cow;

use indexmap::IndexMap;
use nom::{
    branch::alt,
    character::complete::{char, line_ending, not_line_ending, satisfy, space0},
    combinator::{eof, map, peek, recognize},
    multi::{fold_many0, many1_count},
    sequence::{delimited, pair, separated_pair, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Line<'a> {
    Comment(Cow<'a, str>),
    EmptyLine,
    GroupHeader(Cow<'a, str>),
    Entry { key: Key<'a>, value: Value<'a> },
}

type Key<'a> = Cow<'a, str>;
type Value<'a> = Cow<'a, str>;
type EntryMap<'a, 'b> = IndexMap<Key<'a>, Value<'b>>;
type DesktopEntry<'a, 'b, 'c> = IndexMap<Cow<'a, str>, EntryMap<'b, 'c>>;

pub fn parse_desktop_entry(input: &str) -> IResult<&str, DesktopEntry> {
    todo!()
}

fn parse_line(input: &str) -> IResult<&str, Line> {
    terminated(
        alt((parse_comment, parse_empty_line, parse_group_header)),
        alt((line_ending, eof)),
    )(input)
}

/// Parse the comment until the end of the line
fn parse_comment(input: &str) -> IResult<&str, Line> {
    map(recognize(pair(char('#'), not_line_ending)), |comment| {
        Line::Comment(Cow::from(comment))
    })(input)
}

/// Parses an empty line, peeks since the line is handled by [`parse_line`]
fn parse_empty_line(input: &str) -> IResult<&str, Line> {
    map(peek(line_ending), |_| Line::EmptyLine)(input)
}

fn parse_group_header(input: &str) -> IResult<&str, Line> {
    map(
        delimited(
            char('['),
            recognize(many1_count(satisfy(|c| {
                c.is_ascii() && !c.is_control() && c != '[' && c != ']'
            }))),
            char(']'),
        ),
        |header| Line::GroupHeader(Cow::from(header)),
    )(input)
}

fn parse_entry(input: &str) -> IResult<&str, (Key, Value)> {
    separated_pair(parse_key, tuple((space0, char('='), space0)), parse_value)(input)
}

fn parse_key(input: &str) -> IResult<&str, Key> {
    map(
        recognize(many1_count(satisfy(|c| {
            c.is_ascii_alphanumeric() || c == '-'
        }))),
        Cow::from,
    )(input)
}

/// Parse all the characters until the line ending
fn parse_value(input: &str) -> IResult<&str, Value> {
    map(not_line_ending, Cow::from)(input)
}

#[cfg(test)]
mod test {
    use indexmap::indexmap;

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
        assert_eq!(Ok(("\n", Line::EmptyLine)), parse_empty_line("\n"))
    }

    #[test]
    fn shoul_parse_group_header() {
        assert_eq!(
            Ok(("", Line::GroupHeader(Cow::from("header")))),
            parse_group_header("[header]")
        );
    }

    #[test]
    fn shoul_parse_entry() {
        assert_eq!(
            Ok(("", (Cow::from("Ke1"), Cow::from("Value")))),
            parse_entry("Ke1=Value")
        );
    }

    #[test]
    fn shoul_parse_key() {
        assert_eq!(Ok(("", Cow::from("Ke1"))), parse_key("Ke1"));
    }

    #[test]
    fn should_parse_example_file() {
        let example_file = include_str!("../example/file.desktop");

        let (rest, desktop_entry) = parse_desktop_entry(example_file).unwrap();

        assert_eq!("", rest);

        let expected = indexmap! {
            Cow::from("Desktop Entry") => indexmap! {
                Cow::from("Version") => Cow::from("1.0"),
                Cow::from("Type") => Cow::from("Application"),
                Cow::from("Name") => Cow::from("Foo Viewer"),
                Cow::from("Comment") => Cow::from("The best viewer for Foo objects available!"),
                Cow::from("TryExec") => Cow::from("fooview"),
                Cow::from("Exec") => Cow::from("fooview %F"),
                Cow::from("Icon") => Cow::from("fooview"),
                Cow::from("MimeType") => Cow::from("image/x-foo"),
                Cow::from("Actions") => Cow::from("Gallery;Create;"),
            },
            Cow::from("Desktop Action Gallery") => indexmap! {
                Cow::from("Exec") => Cow::from("fooview --gallery"),
                Cow::from("Name") => Cow::from("Browse Gallery"),
            },
            Cow::from("Desktop Action Create") => indexmap! {
                Cow::from("Exec") => Cow::from("fooview --create-new"),
                Cow::from("Name") => Cow::from("Create a new Foo!"),
                Cow::from("Name") => Cow::from("fooview-new"),
            },
        };

        assert_eq!(expected, desktop_entry)
    }
}
