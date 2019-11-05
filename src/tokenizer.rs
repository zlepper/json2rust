use crate::shared::{Error, JsonTokenInfo};

#[derive(Debug, Eq, PartialEq)]
pub struct JsonToken {
    location: JsonTokenInfo,
    token_type: JsonTokenType,
}

impl JsonToken {
    fn new(token_type: JsonTokenType, location: JsonTokenInfo) -> JsonToken {
        JsonToken {
            location,
            token_type,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum JsonTokenType {
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    String(String),
    Float,
    Int,
    Bool,
    Colon,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct TokenizerStringReadingState {
    starting_location: JsonTokenInfo,
    value: String,
    escape_next: bool,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct TokenizerNumberReadingState {
    starting_location: JsonTokenInfo,
    seen_decimal_char: bool,
    seen_exponent: bool,
}

impl TokenizerNumberReadingState {
    fn new(starting_location: JsonTokenInfo) -> TokenizerNumberReadingState {
        TokenizerNumberReadingState {
            starting_location,
            seen_exponent: false,
            seen_decimal_char: false,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum TokenizerState {
    Ready,
    ReadingString(TokenizerStringReadingState),
    ReadingNumber(TokenizerNumberReadingState),
}

pub fn tokenize_json(json: &str) -> Result<Vec<JsonToken>, Error> {
    let mut tokens = Vec::new();

    let mut state = TokenizerState::Ready;

    let mut line_number = 1;
    let mut column_number = 1;

    for (char_index, current_char) in json.chars().enumerate() {
        let index = char_index as i64;
        match state {
            TokenizerState::Ready => {
                let location = JsonTokenInfo::new(line_number, column_number, index);
                match current_char {
                    '{' => tokens.push(JsonToken::new(JsonTokenType::ObjectStart, location)),
                    '}' => tokens.push(JsonToken::new(JsonTokenType::ObjectEnd, location)),
                    '[' => tokens.push(JsonToken::new(JsonTokenType::ArrayStart, location)),
                    ']' => tokens.push(JsonToken::new(JsonTokenType::ArrayEnd, location)),
                    ':' => tokens.push(JsonToken::new(JsonTokenType::Colon, location)),
                    // Don't care about commas
                    ',' => {}
                    '"' => {
                        state = TokenizerState::ReadingString(TokenizerStringReadingState {
                            starting_location: location,
                            escape_next: false,
                            value: "".into(),
                        })
                    }
                    c if c.is_numeric() => {
                        if c == '0' {
                            return Err(Error::NumbersCannotStartWithZero(location));
                        }
                        state = TokenizerState::ReadingNumber(TokenizerNumberReadingState::new(
                            location,
                        ));
                    }
                    ' ' => {}
                    _ => {
                        return Err(Error::UnknownJsonCharacter(location, current_char));
                    }
                }
            }
            TokenizerState::ReadingString(ref s) => {
                if s.escape_next {
                    state = TokenizerState::ReadingString(TokenizerStringReadingState {
                        escape_next: false,
                        value: format!("{}{}", s.value, current_char),
                        ..*s
                    });
                    continue;
                }

                match current_char {
                    '"' => {
                        // End reading this token
                        tokens.push(JsonToken::new(
                            JsonTokenType::String(s.value.clone()),
                            s.starting_location,
                        ));
                        state = TokenizerState::Ready;
                    }
                    '\\' => {
                        state = TokenizerState::ReadingString(TokenizerStringReadingState {
                            escape_next: true,
                            value: s.value.clone(),
                            starting_location: s.starting_location,
                        });
                    }
                    // We don't care about any other specific characters
                    _ => {
                        state = TokenizerState::ReadingString(TokenizerStringReadingState {
                            value: format!("{}{}", s.value, current_char),
                            ..*s
                        });
                    }
                }
            }
            TokenizerState::ReadingNumber(s) => match current_char {
                '.' if s.seen_decimal_char => {
                    return Err(Error::MultipleDecimalSeparators(s.starting_location));
                }
                '.' if s.seen_exponent => {
                    return Err(Error::DecimalAfterExponent(s.starting_location));
                }
                '.' => {
                    state = TokenizerState::ReadingNumber(TokenizerNumberReadingState {
                        seen_decimal_char: true,
                        ..s
                    })
                }
                'e' | 'E' if s.seen_exponent => {
                    return Err(Error::MultipleExponentCharacters(s.starting_location));
                }
                'e' | 'E' => {
                    state = TokenizerState::ReadingNumber(TokenizerNumberReadingState {
                        seen_exponent: true,
                        ..s
                    })
                }
                // This is fine, we just continue parsing it
                c if c.is_numeric() => {}
                ',' | ']' | '}' => {
                    state = end_current_token(&mut tokens, &mut state)?;
                    match current_char {
                        ']' => tokens.push(JsonToken::new(
                            JsonTokenType::ArrayEnd,
                            JsonTokenInfo::new(line_number, column_number, index),
                        )),
                        '}' => tokens.push(JsonToken::new(
                            JsonTokenType::ObjectEnd,
                            JsonTokenInfo::new(line_number, column_number, index),
                        )),
                        _ => {}
                    }
                }
                v => return Err(Error::InvalidNumberCharacter(s.starting_location, v)),
            },
        }
        column_number += 1;
    }

    end_current_token(&mut tokens, &mut state)?;

    Ok(tokens)
}

fn end_current_token(
    mut tokens: &mut Vec<JsonToken>,
    state: &mut TokenizerState,
) -> Result<TokenizerState, Error> {
    println!("Ending state: {:?}", state);
    match state {
        TokenizerState::ReadingString(s) => {
            return Err(Error::UnclosedString(s.starting_location));
        }
        TokenizerState::ReadingNumber(s) => {
            println!("Reading number");
            add_number_to_tokens(&mut tokens, *s);
        }
        // If the tokenizer is simple ready, then we don't really have to do anything
        TokenizerState::Ready => {}
    }
    Ok(TokenizerState::Ready)
}

fn add_number_to_tokens(tokens: &mut Vec<JsonToken>, s: TokenizerNumberReadingState) -> () {
    if s.seen_decimal_char {
        println!("Seen decimal char");
        tokens.push(JsonToken::new(JsonTokenType::Float, s.starting_location));
    } else {
        println!("Not seen decimal");
        tokens.push(JsonToken::new(JsonTokenType::Int, s.starting_location));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_tokenize(json: &str) -> Vec<JsonTokenType> {
        tokenize_json(json)
            .unwrap()
            .into_iter()
            .map(|v| v.token_type)
            .collect()
    }

    #[test]
    fn tokenizes_a_simple_object() {
        let result = simple_tokenize(r#"{}"#);

        assert_eq!(
            result,
            vec![JsonTokenType::ObjectStart, JsonTokenType::ObjectEnd]
        )
    }

    #[test]
    fn tokenizes_a_simple_array() {
        let result = simple_tokenize(r#"[]"#);

        assert_eq!(
            result,
            vec![JsonTokenType::ArrayStart, JsonTokenType::ArrayEnd]
        )
    }

    #[test]
    fn tokenizes_simple_key_value_object() {
        let result = simple_tokenize(r#"{"foo": "bar"}"#);

        assert_eq!(
            result,
            vec![
                JsonTokenType::ObjectStart,
                JsonTokenType::String("foo".into()),
                JsonTokenType::Colon,
                JsonTokenType::String("bar".into()),
                JsonTokenType::ObjectEnd
            ]
        )
    }

    #[test]
    fn tokenizes_simple_string_array() {
        let result = simple_tokenize(r#"["foo"]"#);

        assert_eq!(
            result,
            vec![
                JsonTokenType::ArrayStart,
                JsonTokenType::String("foo".into()),
                JsonTokenType::ArrayEnd
            ]
        )
    }

    #[test]
    fn tokenizes_simple_int_array() {
        let result = simple_tokenize(r#"[42]"#);

        assert_eq!(
            result,
            vec![
                JsonTokenType::ArrayStart,
                JsonTokenType::Int,
                JsonTokenType::ArrayEnd
            ]
        )
    }

    #[test]
    fn tokenizes_simple_float_array() {
        let result = simple_tokenize(r#"[42.5]"#);

        assert_eq!(
            result,
            vec![
                JsonTokenType::ArrayStart,
                JsonTokenType::Float,
                JsonTokenType::ArrayEnd
            ]
        )
    }

    #[test]
    fn tokenizes_multiple_object_key_pairs() {
        let result = simple_tokenize(r#"{"foo": "bar", "baz": "bing"}"#);

        assert_eq!(
            result,
            vec![
                JsonTokenType::ObjectStart,
                JsonTokenType::String("foo".into()),
                JsonTokenType::Colon,
                JsonTokenType::String("bar".into()),
                JsonTokenType::String("baz".into()),
                JsonTokenType::Colon,
                JsonTokenType::String("bing".into()),
                JsonTokenType::ObjectEnd
            ]
        )
    }

    #[test]
    fn error_on_double_decimal() {
        let result = tokenize_json("5.5.5");
        assert_eq!(
            result,
            Err(Error::MultipleDecimalSeparators(JsonTokenInfo::new(
                1, 1, 0,
            )))
        )
    }

    #[test]
    fn error_on_double_exponent() {
        let result = tokenize_json("5e5e5");
        assert_eq!(
            result,
            Err(Error::MultipleExponentCharacters(JsonTokenInfo::new(
                1, 1, 0,
            )))
        )
    }

    #[test]
    fn error_on_decimal_after_exponent() {
        let result = tokenize_json("5e5.5");
        assert_eq!(
            result,
            Err(Error::DecimalAfterExponent(JsonTokenInfo::new(1, 1, 0)))
        )
    }

    #[test]
    fn error_on_unclosed_string() {
        let result = tokenize_json(r#""foo"#);
        assert_eq!(
            result,
            Err(Error::UnclosedString(JsonTokenInfo::new(1, 1, 0)))
        )
    }

    #[test]
    fn error_if_number_starts_with_0() {
        let result = tokenize_json("042");
        assert_eq!(
            result,
            Err(Error::NumbersCannotStartWithZero(JsonTokenInfo::new(
                1, 1, 0,
            )))
        )
    }

    #[test]
    fn nested_objects() {
        let result = simple_tokenize(r#"{"foo": {"bar": "baz"}}"#);

        assert_eq!(
            result,
            vec![
                JsonTokenType::ObjectStart,
                JsonTokenType::String("foo".into()),
                JsonTokenType::Colon,
                JsonTokenType::ObjectStart,
                JsonTokenType::String("bar".into()),
                JsonTokenType::Colon,
                JsonTokenType::String("baz".into()),
                JsonTokenType::ObjectEnd,
                JsonTokenType::ObjectEnd,
            ]
        )
    }
}
