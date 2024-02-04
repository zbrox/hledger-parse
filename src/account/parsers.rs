use winnow::{
    ascii::{space1, till_line_ending},
    combinator::preceded,
    PResult, Parser,
};

pub fn parse_account_directive<'s>(input: &mut &'s str) -> PResult<&'s str> {
    preceded(("account", space1), till_line_ending)
        .verify(|account_name: &str| !account_name.contains("  "))
        .context(winnow::error::StrContext::Label("account name"))
        .parse_next(input)
}
