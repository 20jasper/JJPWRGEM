#![cfg_attr(test, feature(new_range))]

mod error;
mod number;
mod primitive;

use error::{Error, Result};
use jjpwrgem_parse::{Kind, Object, ast::Document};

use crate::primitive::{type_as_primitive, value_as_primitive};

pub fn validate<S: AsRef<str>>(doc: &Document<S>, schema: &Schema) -> Result<()> {
    let root = doc.root();
    match schema {
        // r[impl json-schema-v7.schema.boolean-true]
        Schema::Bool(true) => Ok(()),
        Schema::Bool(false) => Err(Error::FalseRoot { range: root.span() }),
        Schema::Object(ObjectSchema { types: None, .. }) => Ok(()),
        Schema::Object(ObjectSchema {
            types: Some(ty), ..
        }) => {
            let actual = value_as_primitive(root);
            let expected = type_as_primitive(ty);

            if actual.is_subset(expected) {
                Ok(())
            } else {
                let range = root.span();
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

mod keyword {
    pub const TITLE: &str = "title";
    pub const DESCRIPTION: &str = "description";
    pub const TYPE: &str = "type";
}

impl ObjectSchema {
    fn parse<S: AsRef<str>>(object: Object<'_, S>) -> Result<Self> {
        Ok(Self {
            title: string_keyword(object, keyword::TITLE),
            description: string_keyword(object, keyword::DESCRIPTION),
            types: type_keyword(object)?,
        })
    }
}

fn string_keyword<S: AsRef<str>>(object: Object<'_, S>, key: &str) -> Option<String> {
    object.get(key)?.as_str().map(|s| s.raw().to_owned())
}

fn type_keyword<S: AsRef<str>>(object: Object<'_, S>) -> Result<Option<Type>> {
    let Some(value) = object.get(keyword::TYPE) else {
        return Ok(None);
    };
    let ty = value
        .as_str()
        .and_then(|s| Type::from_name(s.raw()))
        .ok_or_else(|| Error::InvalidType {
            range: value.span(),
            found: value.text().to_owned(),
        })?;
    Ok(Some(ty))
}

impl Schema {
    // r[impl json-schema-v7.schema.root]
    pub fn parse<S: AsRef<str>>(doc: &Document<S>) -> Result<Self> {
        let root = doc.root();
        match root.kind() {
            Kind::Bool(b) => Ok(Schema::Bool(b)),
            Kind::Object(object) => Ok(Schema::Object(ObjectSchema::parse(object)?)),
            _ => Err(Error::InvalidSchemaRoot {
                range: root.span(),
                found: value_as_primitive(root),
            }),
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

impl Type {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "boolean" => Some(Self::Bool),
            "string" => Some(Self::String),
            "null" => Some(Self::Null),
            "object" => Some(Self::Object),
            "array" => Some(Self::Array),
            "number" => Some(Self::Number),
            "integer" => Some(Self::Integer),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use jjpwrgem_parse::SyntaxOptions;

    use super::*;
    use crate::primitive::PrimitiveType;

    fn doc(json: &str) -> Document<&str> {
        Document::parse(json, SyntaxOptions::JSON).unwrap()
    }

    fn parse_schema(json: &str) -> Result<Schema> {
        Schema::parse(&Document::parse(json.to_string(), SyntaxOptions::JSON).unwrap())
    }

    fn type_schema(ty: Type) -> Schema {
        Schema::Object(ObjectSchema {
            types: Some(ty),
            ..Default::default()
        })
    }

    mod parse {
        use super::*;

        #[test]
        fn empty_object_has_no_constraints() {
            assert_eq!(
                parse_schema("{}"),
                Ok(Schema::Object(ObjectSchema::default()))
            );
        }

        #[test]
        fn title_and_description() {
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
        fn rejects_non_object_or_boolean_root(#[case] json: &str, #[case] found: PrimitiveType) {
            assert!(matches!(
                parse_schema(json),
                Err(Error::InvalidSchemaRoot { found: actual, .. }) if actual == found
            ));
        }

        // r[verify json-schema-v7.schema.root]
        #[rstest::rstest]
        #[case("true", true)]
        #[case("false", false)]
        fn boolean_schema(#[case] json: &str, #[case] value: bool) {
            assert_eq!(parse_schema(json), Ok(Schema::Bool(value)));
        }

        #[rstest::rstest]
        #[case("boolean", Some(Type::Bool))]
        #[case("string", Some(Type::String))]
        #[case("null", Some(Type::Null))]
        #[case("object", Some(Type::Object))]
        #[case("array", Some(Type::Array))]
        #[case("number", Some(Type::Number))]
        #[case("integer", Some(Type::Integer))]
        #[case("float", None)]
        #[case("Object", None)]
        fn type_from_name(#[case] name: &str, #[case] expected: Option<Type>) {
            assert_eq!(Type::from_name(name), expected);
        }
    }

    mod validate {
        use super::*;

        // r[verify json-schema-v7.type.primitive]
        #[rstest::rstest]
        #[case("true", Type::Bool, true)]
        #[case("false", Type::Bool, true)]
        #[case(r#""string""#, Type::String, true)]
        #[case("null", Type::Null, true)]
        #[case("-42", Type::Integer, true)]
        #[case("[1,2,3]", Type::Array, true)]
        #[case("{}", Type::Object, true)]
        #[case("true", Type::String, false)]
        #[case("false", Type::String, false)]
        #[case("null", Type::Bool, false)]
        #[case("123", Type::Bool, false)]
        #[case("[1,2,3]", Type::Bool, false)]
        #[case("{}", Type::Bool, false)]
        #[case("true", Type::Null, false)]
        #[case("false", Type::Null, false)]
        fn checks_type(#[case] json: &str, #[case] ty: Type, #[case] valid: bool) {
            assert_eq!(validate(&doc(json), &type_schema(ty)).is_ok(), valid);
        }

        #[rstest::rstest]
        #[case("null", Type::String, PrimitiveType::String, PrimitiveType::Null, 0..4)]
        #[case("{}", Type::Array, PrimitiveType::Array, PrimitiveType::Object, 0..2)]
        #[case("[1]", Type::Object, PrimitiveType::Object, PrimitiveType::Array, 0..3)]
        #[case("123.5", Type::Integer, PrimitiveType::Integer, PrimitiveType::Number, 0..5)]
        #[case("true", Type::Number, PrimitiveType::Number, PrimitiveType::Boolean, 0..4)]
        #[case(r#""text""#, Type::Integer, PrimitiveType::Integer, PrimitiveType::String, 0..6)]
        fn mismatch_reports_expected_actual_and_range(
            #[case] json: &str,
            #[case] ty: Type,
            #[case] expected: PrimitiveType,
            #[case] actual: PrimitiveType,
            #[case] range: core::range::Range<usize>,
        ) {
            assert_eq!(
                validate(&doc(json), &type_schema(ty)),
                Err(Error::MismatchedTypes {
                    range,
                    expected,
                    actual,
                })
            );
        }

        // r[verify json-schema-v7.schema.boolean-true]
        #[rstest::rstest]
        #[case("5")]
        #[case(r#""text""#)]
        #[case("null")]
        #[case("[1,2]")]
        fn empty_and_true_schemas_accept_any_value(#[case] json: &str) {
            assert!(validate(&doc(json), &parse_schema("{}").unwrap()).is_ok());
            assert!(validate(&doc(json), &Schema::Bool(true)).is_ok());
        }
    }

    mod end_to_end {
        use super::*;

        // r[verify json-schema-v7.type.primitive]
        #[rstest::rstest]
        #[case(
            r#"{"title": "Port", "description": "TCP port", "type": "integer"}"#,
            "8080",
            true
        )]
        #[case(r#"{"type": "integer"}"#, "1.5", false)]
        #[case(r#"{"type": "object"}"#, "{}", true)]
        #[case(r#"{"type": "object"}"#, "[]", false)]
        fn top_level_type(#[case] schema: &str, #[case] instance: &str, #[case] valid: bool) {
            let schema = parse_schema(schema).unwrap();
            assert_eq!(validate(&doc(instance), &schema).is_ok(), valid);
        }

        #[test]
        fn top_level_unknown_type_is_an_error() {
            assert!(matches!(
                parse_schema(r#"{"type": "float"}"#),
                Err(Error::InvalidType { found, .. }) if found == r#""float""#
            ));
        }
    }
}
