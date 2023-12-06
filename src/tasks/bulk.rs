use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{not_line_ending, tab},
    combinator::{all_consuming, map},
    error::{context, ContextError, ParseError, VerboseError},
    sequence::{preceded, tuple},
    Finish,
};

use super::Error;

pub fn multiple_lines(input: &str) -> Result<Vec<(u64, Action)>, Error> {
    input
        .lines()
        .map(action_from_line)
        .try_fold(vec![], |mut a, x| {
            a.push(x?);
            Ok(a)
        })
}
pub fn action_from_line(input: &str) -> Result<(u64, Action), Error> {
    let (_, res) = line_parser::<VerboseError<&str>>(input)
        .finish()
        .map_err(|e| Error::ParsingLineFailed(Box::new(e.to_string())))?;
    Ok(res)
}

fn line_parser<'a, E>(input: &'a str) -> Result<(&'a str, (u64, Action)), nom::Err<E>>
where
    E: ParseError<&'a str> + ContextError<&'a str>,
{
    all_consuming(tuple((
        context(
            "parsing number at the start of the line",
            nom::character::complete::u64,
        ),
        preceded(
            context("tab before action", tab),
            context(
                "getting action",
                alt((
                    map(
                        tuple((tag("done"), tab, context("title", not_line_ending))),
                        |_| Action::Done,
                    ),
                    map(
                        preceded(
                            tuple((
                                tag("aban"),
                                tab,
                                context(
                                    "title",
                                    take_while(|x: char| !x.eq_ignore_ascii_case(&'\t')),
                                ),
                            )),
                            preceded(tab, context("reason", not_line_ending)),
                        ),
                        |reason: &'a str| Action::Abandon(Some(reason.to_owned())),
                    ),
                    map(
                        tuple((tag("aban"), tab, context("title", not_line_ending))),
                        |_| Action::Abandon(None),
                    ),
                    map(
                        tuple((
                            tag("ignr"),
                            tab,
                            context(
                                "title",
                                take_while(|x: char| !x.eq_ignore_ascii_case(&'\t')),
                            ),
                        )),
                        |_| Action::Ignore,
                    ),
                    map(
                        tuple((
                            tag("back"),
                            tab,
                            context(
                                "title",
                                take_while(|x: char| !x.eq_ignore_ascii_case(&'\t')),
                            ),
                        )),
                        |_| Action::Backlog,
                    ),
                    map(
                        tuple((
                            tag("todo"),
                            tab,
                            context(
                                "title",
                                take_while(|x: char| !x.eq_ignore_ascii_case(&'\t')),
                            ),
                        )),
                        |_| Action::ToDo,
                    ),
                )),
            ),
        ),
    )))(input)
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    Ignore,
    Done,
    ToDo,
    Abandon(Option<String>),
    Backlog,
}
#[cfg(test)]
mod testing {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::ignr("1\tignr\ttask description, done ", (1, Action::Ignore))]
    #[case::done("0\tdone\ttask description", (0, Action::Done))]
    #[case::todo("2\ttodo\ttask description",(2, Action::ToDo))]
    #[case::aban("10\taban\ttask description",(10, Action::Abandon(None)))]
    #[case::aban_with_reason("10\taban\ttask description\tsome reason",(10, Action::Abandon(Some("some reason".to_owned()))))]
    #[case::back("01\tback\ttask description", (1, Action::Backlog))]
    fn bulk_parser_oneline_happy(#[case] input: &str, #[case] expect: (u64, Action)) {
        assert_eq!(action_from_line(input).unwrap(), expect);
    }
    #[rstest]
    #[case::ignr("\tignr\ttask description, done\t ", (1, Action::Ignore))]
    #[case::done("done\ttask description\t", (0, Action::Done))]
    #[case::todo("2\ttodo\ttask description\t0",(2, Action::ToDo))]
    #[case::aban("10 10\taban\ttask description",(10, Action::Abandon(None)))]
    #[case::aban_with_reason("1 0\taban\ttask description\tsome reason",(10, Action::Abandon(Some("some reason".to_owned()))))]
    #[case::back("01 back ttask description", (1, Action::Backlog))]
    #[should_panic]
    fn bulk_parser_oneline_not_happy(#[case] input: &str, #[case] expect: (u64, Action)) {
        assert_eq!(action_from_line(input).unwrap(), expect);
    }
    #[rstest]
    #[case::ignr_multiple("1\tignr\ttask description, done \n0\tdone\ttask description\n10\taban\ttask description\tsome reason\n", vec![(1, Action::Ignore), (0, Action::Done),(10, Action::Abandon(Some("some reason".to_owned())))])]
    #[case::done_single("0\tdone\ttask description\n", vec![(0, Action::Done)])]
    fn bulk_parser_multiline(#[case] input: &str, #[case] expect: Vec<(u64, Action)>) {
        assert_eq!(multiple_lines(input).unwrap(), expect);
    }
}
