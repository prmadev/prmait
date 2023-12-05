use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::char,
    combinator::map,
    error::{context, ContextError, ParseError},
    sequence::{terminated, tuple},
    Parser,
};
use time::{Date, Month, UtcOffset};

use super::day_from_today;

pub fn parse_date(input: &str, offset: UtcOffset) -> Result<Date, Error> {
    let lowered = input.to_lowercase();
    let content = lowered.as_str();
    let (_, action) = date_parser::<nom::error::Error<_>>(content)
        .map_err(|e: nom::Err<_>| Error::ErrorParsingDate(e.to_string()))?;
    match action {
        ParserAction::TimeFromNow(duration) => match duration {
            TimeUnit::Day(count) => Ok(day_from_today(offset, i64::from(count))),
            TimeUnit::Week(count) => Ok(day_from_today(offset, i64::from(count) * 7)), // TODO: make this smarter
            TimeUnit::Month(count) => Ok(day_from_today(offset, i64::from(count) * 30)), // TODO: make this smarter
            TimeUnit::Year(count) => Ok(day_from_today(offset, i64::from(count) * 365)), // TODO: make this smarter
        },
        ParserAction::SpecificDate(y, m, d) => Ok(time::Date::from_calendar_date(y, m, d)?),
    }
}

#[derive(Debug)]
enum ParserAction {
    TimeFromNow(TimeUnit),
    SpecificDate(i32, Month, u8),
}
fn date_parser<'a, E>(content: &'a str) -> Result<(&str, ParserAction), nom::Err<E>>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    alt((
        context("from_now_parser", date_duration_parser),
        context("from_named_parser", named_duration_to_parser),
        context("specific_time_parser", full_date_parser),
    ))
    .parse(content)
}

#[derive(Debug)]
enum TimeUnit {
    Day(u16),
    Week(u16),
    Month(u16),
    Year(u16),
}

fn date_duration_parser<'a, E>(input: &'a str) -> Result<(&'a str, ParserAction), nom::Err<E>>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    alt((
        map(
            terminated(
                nom::character::complete::u16,
                alt((tag_no_case("days"), tag_no_case("day"), tag_no_case("d"))),
            ),
            |count| ParserAction::TimeFromNow(TimeUnit::Day(count)),
        ),
        map(
            terminated(
                nom::character::complete::u16,
                alt((tag_no_case("weeks"), tag_no_case("week"), tag_no_case("w"))),
            ),
            |count| ParserAction::TimeFromNow(TimeUnit::Week(count)),
        ),
        map(
            terminated(
                nom::character::complete::u16,
                alt((
                    tag_no_case("months"),
                    tag_no_case("month"),
                    tag_no_case("m"),
                )),
            ),
            |count| ParserAction::TimeFromNow(TimeUnit::Month(count)),
        ),
        map(
            terminated(
                nom::character::complete::u16,
                alt((tag_no_case("years"), tag_no_case("year"), tag_no_case("y"))),
            ),
            |count| ParserAction::TimeFromNow(TimeUnit::Year(count)),
        ),
    ))
    .parse(input)
}

fn full_date_parser<'a, E>(content: &'a str) -> Result<(&str, ParserAction), nom::Err<E>>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    map(
        tuple((
            context("year_parsing", nom::character::complete::i32),
            context(
                "month_parsing",
                nom::sequence::delimited(
                    alt((char('_'), char('-'), char('.'))),
                    month_extractor,
                    alt((char('_'), char('-'), char('.'))),
                ),
            ),
            context("day_parsing", nom::character::complete::u8),
        )),
        |(y, m, d)| ParserAction::SpecificDate(y, m, d),
    )
    .parse(content)
}

fn month_extractor<'a, E>(content: &'a str) -> Result<(&str, Month), nom::Err<E>>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    alt((
        // these three should go first because other cases, like match 1, will precede, and discard the rest
        map(alt((tag_no_case("10"), tag_no_case("october") /* the longer case should always go first, to match before throwing out the rest*/, tag_no_case("oct"))), |_| {
            Month::October
        }),
        map(alt((tag_no_case("11"), tag_no_case("november"), tag_no_case("nov"))), |_| {
            Month::November
        }),
        map(alt((tag_no_case("12"), tag_no_case("december"), tag_no_case("dec"))), |_| {
            Month::December
        }),
        // end of these three
        map(
            alt((tag_no_case("01"), tag_no_case("1"), tag_no_case("january"), tag_no_case("jan"))),
            |_| Month::January,
        ),
        map(
            alt((tag_no_case("02"), tag_no_case("2"), tag_no_case("february"), tag_no_case("feb"))),
            |_| Month::February,
        ),
        map(alt((tag_no_case("03"), tag_no_case("3"), tag_no_case("march"), tag_no_case("mar"))), |_| {
            Month::March
        }),
        map(alt((tag_no_case("04"), tag_no_case("4"), tag_no_case("april"), tag_no_case("apr"))), |_| {
            Month::April
        }),
        map(alt((tag_no_case("05"), tag_no_case("5"), tag_no_case("may"))), |_| Month::May),
        map(alt((tag_no_case("06"), tag_no_case("6"), tag_no_case("june"), tag_no_case("jun"))), |_| {
            Month::June
        }),
        map(alt((tag_no_case("07"), tag_no_case("7"), tag_no_case("july"), tag_no_case("jul"))), |_| {
            Month::July
        }),
        map(
            alt((tag_no_case("08"), tag_no_case("8"), tag_no_case("august"), tag_no_case("aug"))),
            |_| Month::August,
        ),
        map(
            alt((tag_no_case("09"), tag_no_case("9"), tag_no_case("september"), tag_no_case("sep"))),
            |_| Month::September,
        ),
    ))
    .parse(content)
}

fn named_duration_to_parser<'a, E>(content: &'a str) -> Result<(&str, ParserAction), nom::Err<E>>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    alt((
        map(
            context("today_parser", alt((tag("tod"), tag("today")))),
            |_| ParserAction::TimeFromNow(TimeUnit::Day(0)),
        ),
        map(
            context("tomorrow_parser", alt((tag("tom"), tag("tomorrow")))),
            |_| ParserAction::TimeFromNow(TimeUnit::Day(1)),
        ),
    ))
    .parse(content)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not find any pattern matching that time input")]
    CouldNotFindAnyPattern,
    #[error("could not get time")]
    CreatingTimeFailed(#[from] time::error::ComponentRange),
    #[error("failed in adding time to day")]
    AddingTimeFailed,
    #[error("from must be before after")]
    ToIsNotAfterFrom,
    #[error("got error parsing date: {0}")]
    ErrorParsingDate(String),
}

#[cfg(test)]
mod testing {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use crate::timeutils::{today, tomorrow};
    use rstest::*;
    use time::{Month, UtcOffset};

    const fn is_normal<T: Sized + Send + Sync + Unpin>() {}

    #[test]
    const fn normal_types() {
        is_normal::<Error>();
    }

    #[rstest]
    #[case::parse_underlined_with_zero("1993_04_10" ,time::Date::from_calendar_date(1993, Month::April, 10).unwrap())]
    #[case::parse_underlined_and_dotted("1993_04.10" ,time::Date::from_calendar_date(1993, Month::April, 10).unwrap())]
    #[case::parse_dotted_and_underlined_without_zero("1993.4_10" ,time::Date::from_calendar_date(1993, Month::April, 10).unwrap())]
    #[case::parse_apr_underlined_dotted("1993_apr.10" ,time::Date::from_calendar_date(1993, Month::April, 10).unwrap())]
    #[case::parse_time_dash("1993-04-10" ,time::Date::from_calendar_date(1993, Month::April, 10).unwrap())]
    #[case::parse_april_with_random_capital("1993-apRIl-10" ,time::Date::from_calendar_date(1993, Month::April, 10).unwrap())]
    #[case::parse_tod("tod", today(UtcOffset::UTC))]
    #[case::parse_today("today", today(UtcOffset::UTC))]
    #[case::parse_tom("tom", tomorrow(UtcOffset::UTC))]
    #[case::parse_tomorrow("tomorrow", tomorrow(UtcOffset::UTC))]
    #[case::parse_3days("3days", day_from_today(UtcOffset::UTC, 3))]
    #[case::parse_3days("3day", day_from_today(UtcOffset::UTC, 3))]
    #[case::parse_3days("3d", day_from_today(UtcOffset::UTC, 3))]
    #[case::parse_1weeks("1weeks", day_from_today(UtcOffset::UTC, 7))]
    #[case::parse_1week("1week", day_from_today(UtcOffset::UTC, 7))]
    #[case::parse_1w("1w", day_from_today(UtcOffset::UTC, 7))]
    #[case::parse_3weeks("3weeks", day_from_today(UtcOffset::UTC, 3 * 7))]
    #[case::parse_1week("3week", day_from_today(UtcOffset::UTC, 3 * 7))]
    #[case::parse_1w("3w", day_from_today(UtcOffset::UTC, 3 * 7))]
    #[case::parse_1m("1m", day_from_today(UtcOffset::UTC, 30))]
    #[case::parse_1month("1month", day_from_today(UtcOffset::UTC, 30))]
    #[case::parse_1months("1months", day_from_today(UtcOffset::UTC, 30))]
    #[case::parse_22months("22months", day_from_today(UtcOffset::UTC, 22*30))]
    #[case::parse_22months("22m", day_from_today(UtcOffset::UTC, 22*30))]
    #[case::parse_1years("1years", day_from_today(UtcOffset::UTC, 365))]
    #[case::parse_1year("1year", day_from_today(UtcOffset::UTC, 365))]
    #[case::parse_1y("1y", day_from_today(UtcOffset::UTC, 365))]
    #[case::parse_10y("10y", day_from_today(UtcOffset::UTC, 10*365))]
    #[case::parse_10year("10year", day_from_today(UtcOffset::UTC, 10*365))]
    #[case::parse_10years("10years", day_from_today(UtcOffset::UTC, 10*365))]
    fn parse_date_happy_path(#[case] input: &str, #[case] expected: Date) {
        assert_eq!(parse_date(input, UtcOffset::UTC).unwrap(), expected)
    }
}
