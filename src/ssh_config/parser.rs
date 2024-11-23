use nom::{
    branch::alt,
    bytes::complete::{is_not, tag_no_case},
    character::complete::{alpha1, char, multispace0, newline, space0, space1},
    combinator::{map, opt, value},
    error::ParseError,
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};

#[cfg(test)]
mod test;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Keyword {
    Host,
    Hostname,
    Other(String),
}

impl From<Keyword> for String {
    fn from(keyword: Keyword) -> String {
        match keyword{
            Keyword::Host => "host".to_string(),
            Keyword::Hostname => "hostname".to_string(),
            Keyword::Other(key) => key,
        }
    }
}

fn keyword_string<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Keyword, E> {
    alt((
        value(Keyword::Hostname, tag_no_case("hostname")),
        value(Keyword::Host, tag_no_case("host")),
        map(alpha1, |s: &'a str| Keyword::Other(s.to_lowercase())),
    ))(i)
}

fn argument<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    let not_quote = is_not("\"");

    alt((
        delimited(char('"'), not_quote, char('"')),
        is_not(" \t\n\r"),
    ))(input)
}

fn key_value_separator<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    alt((
        map(delimited(opt(space0), char('='), opt(space0)), |_| "="),
        space1,
    ))(input)
}

fn key_value<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, (Keyword, Vec<&'a str>), E> {
    terminated(
        separated_pair(
            keyword_string,
            key_value_separator,
            separated_list0(space1, argument),
        ),
        many0(newline),
    )(input)
}

pub fn ssh_config_value_parser<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Vec<(Keyword, Vec<&'a str>)>, E> {
    many0(preceded(multispace0, key_value))(input)
}
