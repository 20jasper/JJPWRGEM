#![feature(new_range)]

mod error;
mod primitive;

use error::{Error, Result};
use jjpwrgem_parse::ast::{Document, ObjectEntries};

use crate::primitive::{type_as_primitive, value_as_primitive};

pub fn validate<S: AsRef<str>>(doc: &Document<S>, schema: &Schema) -> Result<()> {
    match schema {
        // r[impl json-schema-v7.schema.boolean-true]
        Schema::Bool(true) => Ok(()),
        Schema::Bool(false) => Err(Error::FalseRoot {
            range: doc.root().range(),
        }),
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

mod keyword {
    pub const TITLE: &str = "title";
    pub const DESCRIPTION: &str = "description";
    pub const TYPE: &str = "type";
}

impl ObjectSchema {
    fn parse<S: AsRef<str>>(doc: &Document<S>, entries: &ObjectEntries) -> Result<Self> {
        Ok(Self {
            title: string_keyword(doc, entries, keyword::TITLE).map(ToOwned::to_owned),
            description: string_keyword(doc, entries, keyword::DESCRIPTION).map(ToOwned::to_owned),
            types: type_keyword(doc, entries)?,
        })
    }
}

fn string_keyword<'a, S: AsRef<str>>(
    doc: &'a Document<S>,
    entries: &ObjectEntries,
    key: &str,
) -> Option<&'a str> {
    doc.get_object_value(entries, key)
        .and_then(|value| doc.as_str(value))
}

fn type_keyword<S: AsRef<str>>(doc: &Document<S>, entries: &ObjectEntries) -> Result<Option<Type>> {
    let Some(value) = doc.get_object_value(entries, keyword::TYPE) else {
        return Ok(None);
    };
    doc.as_str(value)
        .and_then(Type::from_name)
        .map(Some)
        .ok_or_else(|| Error::InvalidType {
            range: value.span(),
            found: doc.slice(value.span()).to_owned(),
        })
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
        if let Some(b) = doc.as_bool(doc.root()) {
            Ok(Schema::Bool(b))
        } else if let Some(entries) = doc.as_object(doc.root()) {
            Ok(Schema::Object(ObjectSchema::parse(doc, entries)?))
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

    #[rstest::rstest]
    #[case("boolean", Type::Bool)]
    #[case("string", Type::String)]
    #[case("null", Type::Null)]
    #[case("object", Type::Object)]
    #[case("array", Type::Array)]
    #[case("number", Type::Number)]
    #[case("integer", Type::Integer)]
    fn type_from_name_maps_each_json_type(#[case] name: &str, #[case] expected: Type) {
        assert_eq!(Type::from_name(name), Some(expected));
    }

    #[test]
    fn type_from_name_rejects_unknown_names() {
        assert_eq!(Type::from_name("float"), None);
        assert_eq!(Type::from_name("Object"), None);
    }

    #[rstest::rstest]
    #[case("null", Type::String, PrimitiveType::String, PrimitiveType::Null, 0, 4)]
    #[case("{}", Type::Array, PrimitiveType::Array, PrimitiveType::Object, 0, 2)]
    #[case("[1]", Type::Object, PrimitiveType::Object, PrimitiveType::Array, 0, 3)]
    #[case(
        "true",
        Type::Number,
        PrimitiveType::Number,
        PrimitiveType::Boolean,
        0,
        4
    )]
    #[case(
        r#""text""#,
        Type::Integer,
        PrimitiveType::Integer,
        PrimitiveType::String,
        0,
        6
    )]
    fn mismatch_reports_expected_actual_and_range(
        #[case] json: &str,
        #[case] ty: Type,
        #[case] expected: PrimitiveType,
        #[case] actual: PrimitiveType,
        #[case] start: usize,
        #[case] end: usize,
    ) {
        let doc = Document::parse(json, SyntaxOptions::JSON).unwrap();
        assert_eq!(
            validate(&doc, &Schema::type_(ty)),
            Err(Error::MismatchedTypes {
                range: core::range::Range { start, end },
                expected,
                actual,
            })
        );
    }

    // r[verify json-schema-v7.type.integer-subset]
    #[rstest::rstest]
    #[case("1e3", true)]
    #[case("1.5e1", true)]
    #[case("1.0e-1", false)]
    #[case("1.5", false)]
    fn integer_is_decided_by_value_not_text(#[case] json: &str, #[case] valid: bool) {
        let doc = Document::parse(json, SyntaxOptions::JSON).unwrap();
        assert_eq!(validate(&doc, &Schema::type_(Type::Integer)).is_ok(), valid);
    }
}
