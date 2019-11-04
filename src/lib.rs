use shared::{Error, JsonTokenInfo};

mod shared;
mod tokenizer;

#[derive(Debug, Eq, PartialEq)]
pub enum JsonValue {
    String,
    Float,
    Int,
    Bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct JsonPair {
    key: String,
    value: Box<JsonStructure>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum JsonStructure {
    Array(Box<JsonStructure>),
    // We have dictionary and object distinct, as we should output them as
    // slightly different types
    Dictionary(JsonPair),
    Object(Vec<JsonPair>),
    Value(JsonValue),
    Unknown,
}


pub fn convert_sample_json(json: &str) -> Result<JsonStructure, Error> {
    //    json.chars()
    Err(Error::InvalidJson {
        location: JsonTokenInfo::new(0, 0, 0),
        message: "Not implemented".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_a_simple_object() {
        let result = convert_sample_json(r#"{"foo": "bar"}"#).expect("Json conversion failed");

        assert_eq!(
            result,
            JsonStructure::Object(vec![JsonPair {
                key: "foo".into(),
                value: Box::new(JsonStructure::Value(JsonValue::String)),
            }])
        )
    }

    #[test]
    fn handles_escaped_qoutes() {
        let result = convert_sample_json(r#"{"foo": "bar\"baz"}"#).expect("Json conversion failed");

        assert_eq!(
            result,
            JsonStructure::Object(vec![JsonPair {
                key: "foo".into(),
                value: Box::new(JsonStructure::Value(JsonValue::String)),
            }])
        )
    }

    #[test]
    fn handles_escaped_backslashes() {
        let result = convert_sample_json(r#"{"foo": "bar\\baz"}"#).expect("Json conversion failed");

        assert_eq!(
            result,
            JsonStructure::Object(vec![JsonPair {
                key: "foo".into(),
                value: Box::new(JsonStructure::Value(JsonValue::String)),
            }])
        )
    }

    #[test]
    fn converts_a_simple_array() {
        let result = convert_sample_json(r#"["foo", "bar"]"#).expect("Json conversion failed");

        assert_eq!(
            result,
            JsonStructure::Array(Box::new(JsonStructure::Value(JsonValue::String)))
        )
    }
}
