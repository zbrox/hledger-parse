use winnow::{
    ascii::space0,
    combinator::{delimited, preceded, terminated},
    token::take_until,
    PResult, Parser,
};

pub fn is_char_minus(char: char) -> bool {
    char == '-'
}

pub fn in_quotes<'s>(input: &mut &'s str) -> PResult<&'s str> {
    delimited(
        terminated('"', space0),
        take_until(0.., '"'),
        preceded(space0, '"'),
    )
    .parse_next(input)
}

pub fn find_space_before_char(input: &str, char: char) -> Option<usize> {
    let char_pos = input.find(char);
    match char_pos {
        Some(pos) => input[..pos].rfind(' '),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::find_space_before_char;

    #[test]
    fn test_split_on_space_before_char() {
        assert_eq!(
            find_space_before_char("before before source:salary", ':'),
            Some(13)
        );
        assert_eq!(
            find_space_before_char("преди преди източник:заплата", ':'),
            Some(21)
        );
        assert_eq!(find_space_before_char("", ':'), None);
    }
}
