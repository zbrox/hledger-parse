use winnow::{
    ascii::{dec_int, dec_uint, space0},
    combinator::{alt, delimited, preceded, terminated},
    token::{take_until, take_while},
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

/// parses a sequence of 0 or more zeros without returning them
pub fn repeated_zero(input: &mut &str) -> PResult<()> {
    let _ = take_while(0.., |c: char| c == '0').parse_next(input)?;
    Ok(())
}

/// parses a u32 with leading zeros
pub fn decu32_leading_zeros(input: &mut &str) -> PResult<u32> {
    alt((
        preceded(repeated_zero, dec_uint::<_, u32, _>),
        repeated_zero.map(|_| 0),
    ))
    .parse_next(input)
}

/// parses a i32 with leading zeros
pub fn deci32_leading_zeros(input: &mut &str) -> PResult<i32> {
    alt((
        preceded(repeated_zero, dec_int::<_, i32, _>),
        repeated_zero.map(|_| 0),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

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

    #[rstest]
    #[case::one_repeated_zero("0", Ok(()))]
    #[case::several_repeated_zero("00000", Ok(()))]
    fn test_repeated_zero_parser(
        #[case] input: &str,
        #[case] expected: Result<(), winnow::error::ErrMode<winnow::error::ContextError>>,
    ) {
        let mut input = input;
        assert_eq!(repeated_zero(&mut input), expected);
        assert_eq!(input, "");
    }

    #[rstest]
    #[case::decu32_leading_only_one_zero("0", Ok(0))]
    #[case::decu32_leading_only_several_zeros("00000", Ok(0))]
    #[case::decu32_leading_one_zero("01", Ok(1))]
    #[case::decu32_leading_several_zeros("000001", Ok(1))]
    fn test_parse_decu32_leading_zeros(
        #[case] input: &str,
        #[case] expected: Result<u32, winnow::error::ErrMode<winnow::error::ContextError>>,
    ) {
        let mut input = input;
        assert_eq!(decu32_leading_zeros(&mut input), expected);
        assert_eq!(input, "");
    }

    #[rstest]
    #[case::deci32_leading_only_one_zero("0", Ok(0))]
    #[case::deci32_leading_only_several_zeros("00000", Ok(0))]
    #[case::deci32_leading_one_zero("01", Ok(1))]
    #[case::deci32_leading_several_zeros("000001", Ok(1))]
    fn test_parse_deci32_leading_zeros(
        #[case] input: &str,
        #[case] expected: Result<i32, winnow::error::ErrMode<winnow::error::ContextError>>,
    ) {
        let mut input = input;
        assert_eq!(deci32_leading_zeros(&mut input), expected);
        assert_eq!(input, "");
    }
}
