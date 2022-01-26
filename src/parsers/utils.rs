use nom::character::{is_digit, is_newline};

pub fn is_char_newline(char: char) -> bool {
    is_newline(char as u8)
}

pub fn is_char_digit(char: char) -> bool {
    is_digit(char as u8)
}
