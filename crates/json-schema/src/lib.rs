#![feature(new_range)]

use jjpwrgem_parse::ast::{Document, Value};

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
        /// schema root must be a boolean or object. found {found}
        InvalidSchemaRoot {
            range: Range<usize>,
            found: PrimitiveType,
        },
    }

    impl Error {
        pub fn range(&self) -> &Range<usize> {
            match self {
                Error::MismatchedTypes { range, .. } | Error::InvalidSchemaRoot { range, .. } => {
                    range
                }
            }
        }
    }

    impl std::error::Error for Error {}

    pub type Result<T> = std::result::Result<T, Error>;
}

use error::{Error, Result};

use crate::primitive::{type_as_primitive, value_as_primitive};

mod primitive {

    use std::fmt;

    use jjpwrgem_parse::ast::{Document, Value};

    use crate::Type;

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

    pub fn type_as_primitive(ty: &Type) -> PrimitiveType {
        match ty {
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
    match schema {
        // r[impl json-schema-v7.schema.boolean-true]
        Schema::Bool(true) => Ok(()),
        Schema::Bool(false) => todo!(),
        Schema::Object(ObjectSchema { types: None, .. }) => Ok(()),
        Schema::Object(ObjectSchema {
            types: Some(ty), ..
        }) => {
            let actual = value_as_primitive(doc.root(), doc);
            let expected = type_as_primitive(ty);

            if actual.is_subset(expected) {
                Ok(())
            } else {
                let range = doc.root().range();
                Err(Error::MismatchedTypes {
                    range,
                    expected,
                    actual,
                })
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Schema {
    Bool(bool),
    Object(ObjectSchema),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ObjectSchema {
    title: Option<String>,
    description: Option<String>,
    types: Option<Type>,
}

impl Schema {
    fn type_(type_: Type) -> Self {
        Self::Object(ObjectSchema {
            types: Some(type_),
            ..ObjectSchema::default()
        })
    }

    // r[impl json-schema-v7.schema.root]
    pub fn parse<S: AsRef<str>>(doc: &Document<S>) -> Result<Self> {
        if let Value::Boolean(_, b) = doc.root() {
            Ok(Schema::Bool(*b))
        } else if let Value::Object(..) = doc.root() {
            todo!()
        } else {
            Err(Error::InvalidSchemaRoot {
                range: doc.root().range(),
                found: value_as_primitive(doc.root(), doc),
            })
        }
    }
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

    fn parse_schema(json: &str) -> Result<Schema> {
        let doc = Document::parse(json.to_string(), SyntaxOptions::JSON).unwrap();
        Schema::parse(&doc)
    }

    #[test]
    fn parse_empty_object_has_no_constraints() {
        assert_eq!(
            parse_schema("{}"),
            Ok(Schema::Object(ObjectSchema::default()))
        );
    }

    #[rstest::rstest]
    #[case("123")]
    #[case(r#""text""#)]
    #[case("null")]
    #[case("[1,2]")]
    fn empty_schema_accepts_any_value(#[case] json: &str) {
        let schema = parse_schema("{}").unwrap();
        let doc = Document::parse(json, SyntaxOptions::JSON).unwrap();
        assert!(validate(&doc, &schema).is_ok());
    }

    #[test]
    fn parse_title_and_description() {
        assert_eq!(
            parse_schema(r#"{"title": "Port", "description": "TCP port"}"#),
            Ok(Schema::Object(ObjectSchema {
                title: Some("Port".to_string()),
                description: Some("TCP port".to_string()),
                types: None,
            }))
        );
    }

    // r[verify json-schema-v7.schema.root]
    #[rstest::rstest]
    #[case("5", PrimitiveType::Integer)]
    #[case(r#""text""#, PrimitiveType::String)]
    #[case("null", PrimitiveType::Null)]
    #[case("[1,2]", PrimitiveType::Array)]
    fn parse_rejects_non_object_or_boolean_root(#[case] json: &str, #[case] found: PrimitiveType) {
        assert!(matches!(
            parse_schema(json),
            Err(Error::InvalidSchemaRoot { found: actual, .. }) if actual == found
        ));
    }

    // r[verify json-schema-v7.schema.root]
    #[test]
    fn parse_true_boolean_schema() {
        assert_eq!(parse_schema("true"), Ok(Schema::Bool(true)));
    }

    // r[verify json-schema-v7.schema.root]
    #[test]
    fn parse_false_boolean_schema() {
        assert_eq!(parse_schema("false"), Ok(Schema::Bool(false)));
    }

    // r[verify json-schema-v7.schema.boolean-true]
    #[rstest::rstest]
    #[case("5")]
    #[case(r#""text""#)]
    #[case("null")]
    #[case("[1,2]")]
    fn true_schema_accepts_any_value(#[case] json: &str) {
        let doc = Document::parse(json, SyntaxOptions::JSON).unwrap();
        assert!(validate(&doc, &Schema::Bool(true)).is_ok());
    }
}
