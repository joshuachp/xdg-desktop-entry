use std::{borrow::Cow, cell::Cell};

use indexmap::IndexMap;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, not_line_ending, satisfy, space0, space1},
    combinator::{cut, eof, map, map_parser, opt, peek, recognize, value, verify},
    multi::{fold_many0, many1_count},
    number::complete::float,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

const ESCAPE_CHAR: char = '\\';

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key<'a> {
    Simple(Cow<'a, str>),
    Localized {
        key: Cow<'a, str>,
        locale: Locale<'a>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale<'a> {
    lang: Cow<'a, str>,
    country: Option<Cow<'a, str>>,
    encoding: Option<Cow<'a, str>>,
    modifier: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    LocaleString(Cow<'a, str>),
    // TODO: parse icon-string
    // IconString(Cow<'a, str>),
    Boolean(bool),
    Numeric(f32),
}

impl<'a> Eq for Value<'a> {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Line<'a> {
    Comment(Cow<'a, str>),
    EmptyLine { white_space: Option<Cow<'a, str>> },
    GroupHeader(Cow<'a, str>),
    Entry { key: Key<'a>, value: Value<'a> },
}

struct Group<'a> {
    header: Cow<'a, str>,
    entries: EntryMap<'a, 'a>,
}

#[cfg(feature = "keep-comments")]
#[derive(Debug, Clone, PartialEq, Eq)]
enum Comment<'a> {
    Comment(Cow<'a, str>),
    EmptyLine { white_space: Option<Cow<'a, str>> },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopEntry<'a> {
    groups: IndexMap<Cow<'a, str>, EntryMap<'a, 'a>>,
    #[cfg(feature = "keep-comments")]
    comments: IndexMap<usize, Comment<'a>>,
}

pub type EntryMap<'a, 'b> = IndexMap<Key<'a>, Value<'b>>;

/// Parses a desktop file.
///
/// # Errors
///
/// Invalid or malformed desktop file.
pub fn parse_desktop_entry(input: &str) -> IResult<&str, DesktopEntry> {
    let has_entry = Cell::new(true);

    terminated(
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
                map_document_line,
            ),
            |(mut document, group, _)| {
                if let Some(group) = group {
                    document.groups.insert(group.header, group.entries);
                }

                document
            },
        ),
        eof,
    )(input)
}

#[cfg(feature = "keep-comments")]
fn map_document_line<'a>(
    (mut document, mut group, count): (DesktopEntry<'a>, Option<Group<'a>>, usize),
    line: Line<'a>,
) -> (DesktopEntry<'a>, Option<Group<'a>>, usize) {
    match line {
        Line::Comment(comment) => {
            document.comments.insert(count, Comment::Comment(comment));
        }
        Line::EmptyLine { white_space } => {
            document
                .comments
                .insert(count, Comment::EmptyLine { white_space });
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
}

#[cfg(not(feature = "keep-comments"))]
fn map_document_line<'a>(
    (mut document, mut group, count): (DesktopEntry<'a>, Option<Group<'a>>, usize),
    line: Line<'a>,
) -> (DesktopEntry<'a>, Option<Group<'a>>, usize) {
    match line {
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
        Line::Comment(_) | Line::EmptyLine { .. } => {}
    }

    (document, group, count + 1)
}

fn parse_line(input: &str) -> IResult<&str, Line> {
    terminated(
        alt((
            map(parse_comment, Line::Comment),
            map(parse_group_header, Line::GroupHeader),
            map(parse_entry, |(key, value)| Line::Entry { key, value }),
            map(parse_empty_line, |white_space| Line::EmptyLine {
                white_space,
            }),
        )),
        parse_end_of_line,
    )(input)
}

fn parse_end_of_line(input: &str) -> IResult<&str, &str> {
    alt((line_ending, eof))(input)
}

/// Parse the comment until the end of the line
fn parse_comment(input: &str) -> IResult<&str, Cow<str>> {
    map(recognize(pair(char('#'), not_line_ending)), Cow::from)(input)
}

/// Parses an empty line, peeks since the line is handled by [`parse_line`].
///
/// It will consider lines with only whitespace as empty lines.
fn parse_empty_line(input: &str) -> IResult<&str, Option<Cow<str>>> {
    alt((
        terminated(
            map(space1, |white_space| Some(Cow::from(white_space))),
            peek(parse_end_of_line),
        ),
        map(peek(line_ending), |_| None),
    ))(input)
}

fn parse_group_header(input: &str) -> IResult<&str, Cow<str>> {
    map(
        delimited(
            char('['),
            // Fail for missing header content
            recognize(cut(many1_count(satisfy(|c| {
                c.is_ascii() && !c.is_control() && c != '[' && c != ']'
            })))),
            // If an ope `[` is not close fail the parser
            cut(char(']')),
        ),
        Cow::from,
    )(input)
}

fn parse_entry(input: &str) -> IResult<&str, (Key, Value)> {
    separated_pair(parse_key, tuple((space0, char('='), space0)), parse_value)(input)
}

fn parse_key(input: &str) -> IResult<&str, Key> {
    map(
        pair(
            parse_key_part,
            opt(delimited(char('['), parse_key_locale, char(']'))),
        ),
        |(key, opt_locale)| match opt_locale {
            Some(locale) => Key::Localized { key, locale },
            None => Key::Simple(key),
        },
    )(input)
}

fn parse_key_locale(input: &str) -> IResult<&str, Locale> {
    map(
        tuple((
            parse_key_part,
            opt(preceded(char('_'), parse_key_part)),
            opt(preceded(char('.'), parse_key_part)),
            opt(preceded(char('@'), parse_key_part)),
        )),
        |(lang, country, encoding, modifier)| Locale {
            lang,
            country,
            encoding,
            modifier,
        },
    )(input)
}

fn parse_key_part(input: &str) -> IResult<&str, Cow<str>> {
    map(
        recognize(many1_count(satisfy(|c| {
            c.is_ascii_alphanumeric() || c == '-'
        }))),
        Cow::from,
    )(input)
}

/// Parse all the characters until the line ending
fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        map(parse_boolean, Value::Boolean),
        map(parse_numeric, Value::Numeric),
        map(parse_string, Value::String),
        map(parse_local_string, Value::LocaleString),
    ))(input)
}

fn escaped_chars(input: char) -> Option<&'static str> {
    let escaped = match input {
        's' => " ",
        'n' => "\n",
        't' => "\t",
        'r' => "\r",
        '\\' => "\\",
        ';' => ";",
        _ => {
            return None;
        }
    };

    Some(escaped)
}

fn parse_escaped_string(input: &str) -> IResult<&str, Cow<str>> {
    let mut iter = input.chars().enumerate();

    while let Some((i, c)) = iter.next() {
        if c == ESCAPE_CHAR {
            let escaped = iter
                .next()
                .and_then(|(_, escaped)| escaped_chars(escaped))
                .ok_or_else(|| {
                    nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Escaped,
                    ))
                })?;

            let mut escaped_string = String::with_capacity(input.len());
            escaped_string.push_str(&input[..i]);
            escaped_string.push_str(escaped);

            let mut iter = input[i + 2..].chars();
            while let Some(c) = iter.next() {
                if c == ESCAPE_CHAR {
                    let escaped = iter.next().and_then(escaped_chars).ok_or_else(|| {
                        nom::Err::Error(nom::error::Error::new(
                            input,
                            nom::error::ErrorKind::Escaped,
                        ))
                    })?;

                    escaped_string.push_str(escaped);
                } else {
                    escaped_string.push(c);
                }
            }

            return Ok(("", Cow::Owned(escaped_string)));
        }
    }

    Ok(("", Cow::Borrowed(input)))
}

fn parse_string(input: &str) -> IResult<&str, Cow<str>> {
    map(
        verify(
            map_parser(not_line_ending, cut(parse_escaped_string)),
            str::is_ascii,
        ),
        Cow::from,
    )(input)
}

fn parse_local_string(input: &str) -> IResult<&str, Cow<str>> {
    map(
        map_parser(not_line_ending, cut(parse_escaped_string)),
        Cow::from,
    )(input)
}

fn parse_boolean(input: &str) -> IResult<&str, bool> {
    map_parser(
        not_line_ending,
        alt((value(true, tag("true")), value(false, tag("false")))),
    )(input)
}

fn parse_numeric(input: &str) -> IResult<&str, f32> {
    map_parser(not_line_ending, float)(input)
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
        assert_eq!(Ok(("\n", None)), parse_empty_line("\n"))
    }

    #[test]
    fn shoul_parse_empty_line_whitespace() {
        assert_eq!(Ok(("\n", Some(Cow::from("  ")))), parse_empty_line("  \n"))
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
            Ok((
                "",
                (
                    Key::Simple(Cow::from("Ke1")),
                    Value::String(Cow::from("Value"))
                )
            )),
            parse_entry("Ke1=Value")
        );
    }

    #[test]
    fn shoul_parse_key() {
        assert_eq!(Ok(("", Key::Simple(Cow::from("Ke1")))), parse_key("Ke1"));
    }

    fn example_file_groups() -> IndexMap<Cow<'static, str>, EntryMap<'static, 'static>> {
        indexmap! {
            Cow::from("Desktop Entry") => indexmap! {
                Key::Simple(Cow::from("Version")) => Value::Numeric(1.0),
                Key::Simple(Cow::from("Type")) => Value::String(Cow::from("Application")),
                Key::Simple(Cow::from("Name")) => Value::String(Cow::from("Foo Viewer")),
                Key::Simple(Cow::from("Comment")) => Value::String(Cow::from("The best viewer for Foo objects available!")),
                Key::Simple(Cow::from("TryExec")) => Value::String(Cow::from("fooview")),
                Key::Simple(Cow::from("Exec")) => Value::String(Cow::from("fooview %F")),
                Key::Simple(Cow::from("Icon")) => Value::String(Cow::from("fooview")),
                Key::Simple(Cow::from("MimeType")) => Value::String(Cow::from("image/x-foo;")),
                Key::Simple(Cow::from("Actions")) => Value::String(Cow::from("Gallery;Create;")),
            },
            Cow::from("Desktop Action Gallery") => indexmap! {
                Key::Simple(Cow::from("Exec")) => Value::String(Cow::from("fooview --gallery")),
                Key::Simple(Cow::from("Name")) => Value::String(Cow::from("Browse Gallery")),
            },
            Cow::from("Desktop Action Create") => indexmap! {
                Key::Simple(Cow::from("Exec")) => Value::String(Cow::from("fooview --create-new")),
                Key::Simple(Cow::from("Name")) => Value::String(Cow::from("Create a new Foo!")),
                Key::Simple(Cow::from("Icon")) => Value::String(Cow::from("fooview-new")),
            },
        }
    }

    #[cfg(feature = "keep-comments")]
    #[test]
    fn should_parse_example_file_with_comments() {
        let example_file = include_str!("../example/file.desktop");

        let (rest, desktop_entry) = parse_desktop_entry(example_file).unwrap();

        assert_eq!("", rest);

        let expected = DesktopEntry {
            groups: example_file_groups(),
            comments: indexmap! {
                0 => Comment::Comment(Cow::from("# Example file from the spec")),
                11 => Comment::EmptyLine{white_space:None},
                15 => Comment::EmptyLine{white_space: None},
            },
        };

        assert_eq!(expected, desktop_entry)
    }

    #[cfg(not(feature = "keep-comments"))]
    #[test]
    fn should_parse_example_file_with_comments() {
        let example_file = include_str!("../example/file.desktop");

        let (rest, desktop_entry) = parse_desktop_entry(example_file).unwrap();

        assert_eq!("", rest);

        let expected = DesktopEntry {
            groups: example_file_groups(),
        };

        assert_eq!(expected, desktop_entry)
    }

    #[test]
    fn should_parse_string() {
        assert_eq!(Ok(("", Cow::from("foo bar"))), parse_string("foo bar"));

        assert_eq!(Ok(("", Cow::from("foo 'bar'"))), parse_string("foo 'bar'"));
    }

    #[test]
    fn should_parse_escaped_string() {
        assert_eq!(Ok(("", Cow::from("foo \nbar"))), parse_string("foo \\nbar"));

        assert_eq!(
            Ok(("", Cow::from("foo \t bar"))),
            parse_string("foo \\t\\sbar")
        );

        assert_eq!(Ok(("", Cow::from("foo;bar"))), parse_string("foo\\;bar"));
    }

    #[test]
    fn should_parse_value() {
        assert_eq!(
            Ok(("", Value::String(Cow::from("foo \nbar")))),
            parse_value("foo \\nbar")
        );

        assert_eq!(Ok(("\nas", Value::Boolean(true))), parse_value("true\nas"));
        assert_eq!(
            Ok(("\nas", Value::Boolean(false))),
            parse_value("false\nas")
        );

        assert_eq!(Ok(("\nas", Value::Numeric(1.))), parse_value("1\nas"));
        assert_eq!(Ok(("\nas", Value::Numeric(4.2))), parse_value("4.20\nas"));
        // FIX: this is will not pass
        // assert_eq!(Ok(("\nas", Value::Numeric(4.2))), parse_value("4,20\nas"));
    }
}
