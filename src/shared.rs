#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct JsonTokenInfo {
    line: i64,
    column: i64,
    char: i64,
}

impl JsonTokenInfo {
    pub fn new(line: i64, column: i64, char: i64) -> JsonTokenInfo {
        JsonTokenInfo { line, column, char }
    }
}
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    MultipleDecimalSeparators(JsonTokenInfo),
    DecimalAfterExponent(JsonTokenInfo),
    InvalidNumberCharacter(JsonTokenInfo, char),
    MultipleExponentCharacters(JsonTokenInfo),
    UnknownJsonCharacter(JsonTokenInfo, char),
    UnclosedString(JsonTokenInfo),
    NumbersCannotStartWithZero(JsonTokenInfo),
    InvalidJson {
        location: JsonTokenInfo,
        message: String,
    },
}
