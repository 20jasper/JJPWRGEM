#![feature(new_range)]

use core::default;

use jjpwrgem_parse::ast::Document;

mod error {
    use core::range::Range;

    use displaydoc::Display;

    use crate::primitive::PrimitiveType;

    #[derive(Debug, Display, PartialEq, Eq, Clone)]
    pub enum Error {
        /// mismatched types. expected {expected} but found {actual}
        MismatchedTypes {
            range: Range<usize>,
            expected: PrimitiveType,
            actual: PrimitiveType,
        },
    }

    impl Error {
        pub fn range(&self) -> &Range<usize> {
            match self {
                Error::MismatchedTypes { range, .. } => range,
            }
        }
    }

    impl std::error::Error for Error {}

    pub type Result<T> = std::result::Result<T, Error>;
}

use error::{Error, Result};

use crate::primitive::{schema_as_primitive, value_as_primitive};

mod primitive {

    use std::fmt;

    use jjpwrgem_parse::ast::{Document, Value};

    use crate::{Schema, Type};

    // r[json-schema-v7.type.primitive]
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum PrimitiveType {
        Null,
        Boolean,
        String,
        Number,
        Integer,
        Array,
        Object,
    }

    impl PrimitiveType {
        // r[json-schema-v7.type.integer-subset]
        pub fn is_subset(self, superset: PrimitiveType) -> bool {
            match (self, superset) {
                (PrimitiveType::Integer, PrimitiveType::Number) => true,
                (lhs, rhs) => lhs == rhs,
            }
        }
    }

    impl fmt::Display for PrimitiveType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let name = match self {
                PrimitiveType::Null => "null",
                PrimitiveType::Boolean => "boolean",
                PrimitiveType::String => "string",
                PrimitiveType::Number => "number",
                PrimitiveType::Integer => "integer",
                PrimitiveType::Array => "array",
                PrimitiveType::Object => "object",
            };
            write!(f, "{name}")
        }
    }

    pub fn value_as_primitive<S: AsRef<str>>(value: &Value, doc: &Document<S>) -> PrimitiveType {
        match value {
            Value::Null(_) => PrimitiveType::Null,
            Value::Boolean(_, _) => PrimitiveType::Boolean,
            Value::String(_) => PrimitiveType::String,
            Value::Number { mantissa, .. } => {
                // "integer" which matches any number with a zero fractional part
                // https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.1
                let s = doc.slice(*mantissa);
                if s.find('.')
                    .is_none_or(|i| s[i + 1..].chars().all(|c| c == '0'))
                {
                    PrimitiveType::Integer
                } else {
                    PrimitiveType::Number
                }
            }
            Value::Array(_, _) => PrimitiveType::Array,
            Value::Object(_, _) => PrimitiveType::Object,
        }
    }

    pub fn schema_as_primitive(schema: &Schema) -> PrimitiveType {
        match &schema.type_ {
            Type::Bool => PrimitiveType::Boolean,
            Type::String => PrimitiveType::String,
            Type::Null => PrimitiveType::Null,
            Type::Object => PrimitiveType::Object,
            Type::Array => PrimitiveType::Array,
            Type::Number => PrimitiveType::Number,
            Type::Integer => PrimitiveType::Integer,
        }
    }
}

pub fn validate<S: AsRef<str>>(doc: &Document<S>, schema: &Schema) -> Result<()> {
    let actual = value_as_primitive(doc.root(), doc);
    let expected = schema_as_primitive(schema);

    if !actual.is_subset(expected) {
        let range = doc.root().range();
        Err(Error::MismatchedTypes {
            range,
            expected,
            actual,
        })
    } else {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Schema {
    type_: Type,
    title: Option<String>,
    description: Option<String>,
}

impl Schema {
    pub fn new(type_: Type, title: Option<String>, description: Option<String>) -> Self {
        Self {
            type_,
            title,
            description,
        }
    }

    pub fn type_(type_: Type) -> Self {
        Self {
            type_,
            title: None,
            description: None,
        }
    }

    // // todo maybe generic?
    // // todo special case raw bits
    // pub fn parse(doc: Document<String>) -> Self {
    //     let val = doc.root();

    // }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Bool,
    String,
    Null,
    Object,
    Array,
    Number,
    Integer,
}

#[cfg(test)]
mod tests {
    use jjpwrgem_parse::SyntaxOptions;

    use super::*;
    use crate::primitive::PrimitiveType;

    // r[verify json-schema-v7.type.primitive]
    #[rstest::rstest]
    #[case("true", Schema::type_(Type::Bool))]
    #[case("false", Schema::type_(Type::Bool))]
    #[case(r#""string""#, Schema::type_(Type::String))]
    #[case("null", Schema::type_(Type::Null))]
    #[case("-42", Schema::type_(Type::Integer))]
    #[case("[1,2,3]", Schema::type_(Type::Array))]
    #[case("{}", Schema::type_(Type::Object))]
    fn it_works(#[case] json: &str, #[case] schema: Schema) {
        let doc = Document::parse(json, SyntaxOptions::JSON).unwrap();
        assert!(validate(&doc, &schema).is_ok());
    }

    // r[verify json-schema-v7.type.integer-subset]
    #[rstest::rstest]
    #[case("123.0", Schema::type_(Type::Number))]
    #[case("123.0", Schema::type_(Type::Integer))]
    #[case("123", Schema::type_(Type::Number))]
    #[case("123", Schema::type_(Type::Integer))]
    fn integer_is_subset_of_number(#[case] json: &str, #[case] schema: Schema) {
        let doc = Document::parse(json, SyntaxOptions::JSON).unwrap();
        assert!(validate(&doc, &schema).is_ok());
    }

    #[test]
    fn number_is_not_subset_of_integer() {
        let doc = Document::parse("123.5", SyntaxOptions::JSON).unwrap();
        assert_eq!(
            validate(&doc, &Schema::type_(Type::Integer)),
            Err(Error::MismatchedTypes {
                range: 0..5,
                expected: PrimitiveType::Integer,
                actual: PrimitiveType::Number
            })
        );
    }

    #[rstest::rstest]
    #[case("true", Schema::type_(Type::String))]
    #[case("false", Schema::type_(Type::String))]
    #[case("null", Schema::type_(Type::Bool))]
    #[case("123", Schema::type_(Type::Bool))]
    #[case("[1,2,3]", Schema::type_(Type::Bool))]
    #[case("{}", Schema::type_(Type::Bool))]
    #[case("true", Schema::type_(Type::Null))]
    #[case("false", Schema::type_(Type::Null))]
    fn it_doesnt_work(#[case] json: &str, #[case] schema: Schema) {
        let doc = Document::parse(json, SyntaxOptions::JSON).unwrap();
        assert!(validate(&doc, &schema).is_err());
    }
}
