use std::{borrow::Cow, cell::Cell};

use indexmap::IndexMap;
use nom::{
    branch::alt,
    character::complete::{char, line_ending, not_line_ending, satisfy, space0},
    combinator::{eof, map, peek, recognize, verify},
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

struct Group<'a> {
    header: Cow<'a, str>,
    entries: EntryMap<'a, 'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Comment<'a> {
    Comment(Cow<'a, str>),
    EmptyLine,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopEntry<'a> {
    groups: IndexMap<Cow<'a, str>, EntryMap<'a, 'a>>,
    comments: IndexMap<usize, Comment<'a>>,
}

type Key<'a> = Cow<'a, str>;
type Value<'a> = Cow<'a, str>;
pub type EntryMap<'a, 'b> = IndexMap<Key<'a>, Value<'b>>;

pub fn parse_desktop_entry(input: &str) -> IResult<&str, DesktopEntry> {
    let has_entry = Cell::new(true);

    map(
        fold_many0(
            verify(parse_line, move |line| match line {
                Line::GroupHeader(_) => {
                    has_entry.set(true);

                    true
                }
                Line::Entry { .. } => has_entry.get(),
                _ => true,
            }),
            || (DesktopEntry::default(), None::<Group>, 0usize),
            |(mut document, mut group, count), line| {
                match line {
                    Line::Comment(comment) => {
                        document.comments.insert(count, Comment::Comment(comment));
                    }
                    Line::EmptyLine => {
                        document.comments.insert(count, Comment::EmptyLine);
                    }
                    Line::GroupHeader(header) => {
                        let old_group = group.replace(Group {
                            header,
                            entries: EntryMap::new(),
                        });

                        if let Some(group) = old_group {
                            document.groups.insert(group.header, group.entries);
                        }
                    }
                    Line::Entry { key, value } => {
                        group.as_mut().unwrap().entries.insert(key, value);
                    }
                }

                (document, group, count + 1)
            },
        ),
        |(mut document, group, _)| {
            if let Some(group) = group {
                document.groups.insert(group.header, group.entries);
            }

            document
        },
    )(input)
}

fn parse_line(input: &str) -> IResult<&str, Line> {
    terminated(
        alt((
            map(parse_comment, Line::Comment),
            map(peek_empty_line, |_| Line::EmptyLine),
            map(parse_group_header, Line::GroupHeader),
            map(parse_entry, |(key, value)| Line::Entry { key, value }),
        )),
        alt((line_ending, eof)),
    )(input)
}

/// Parse the comment until the end of the line
fn parse_comment(input: &str) -> IResult<&str, Cow<str>> {
    map(recognize(pair(char('#'), not_line_ending)), Cow::from)(input)
}

/// Parses an empty line, peeks since the line is handled by
fn peek_empty_line(input: &str) -> IResult<&str, &str> {
    peek(line_ending)(input)
}

fn parse_group_header(input: &str) -> IResult<&str, Cow<str>> {
    map(
        delimited(
            char('['),
            recognize(many1_count(satisfy(|c| {
                c.is_ascii() && !c.is_control() && c != '[' && c != ']'
            }))),
            char(']'),
        ),
        Cow::from,
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
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn shoul_parse_comment() {
        assert_eq!(Ok(("\n", Cow::from("# Code"))), parse_comment("# Code\n"))
    }

    #[test]
    fn shoul_parse_empty_comment() {
        assert_eq!(Ok(("", Cow::from("#"))), parse_comment("#"))
    }

    #[test]
    fn shoul_parse_empty_line() {
        assert_eq!(Ok(("\n", "\n")), peek_empty_line("\n"))
    }

    #[test]
    fn shoul_parse_group_header() {
        assert_eq!(
            Ok(("", Cow::from("header"))),
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

        let expected = DesktopEntry {
            groups: indexmap! {
                Cow::from("Desktop Entry") => indexmap! {
                    Cow::from("Version") => Cow::from("1.0"),
                    Cow::from("Type") => Cow::from("Application"),
                    Cow::from("Name") => Cow::from("Foo Viewer"),
                    Cow::from("Comment") => Cow::from("The best viewer for Foo objects available!"),
                    Cow::from("TryExec") => Cow::from("fooview"),
                    Cow::from("Exec") => Cow::from("fooview %F"),
                    Cow::from("Icon") => Cow::from("fooview"),
                    Cow::from("MimeType") => Cow::from("image/x-foo;"),
                    Cow::from("Actions") => Cow::from("Gallery;Create;"),
                },
                Cow::from("Desktop Action Gallery") => indexmap! {
                    Cow::from("Exec") => Cow::from("fooview --gallery"),
                    Cow::from("Name") => Cow::from("Browse Gallery"),
                },
                Cow::from("Desktop Action Create") => indexmap! {
                    Cow::from("Exec") => Cow::from("fooview --create-new"),
                    Cow::from("Name") => Cow::from("Create a new Foo!"),
                    Cow::from("Icon") => Cow::from("fooview-new"),
                },
            },
            comments: indexmap! {
                0 => Comment::Comment(Cow::from("# Example file from the spec")),
                11 => Comment::EmptyLine,
                15 => Comment::EmptyLine,
            },
        };

        assert_eq!(expected, desktop_entry)
    }
}
