use std::fmt;

use jjpwrgem_parse::{Kind, Node};

use crate::{Type, number::Decimal};

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

pub fn value_as_primitive(node: Node<'_>) -> PrimitiveType {
    match node.kind() {
        Kind::Null => PrimitiveType::Null,
        Kind::Bool(_) => PrimitiveType::Boolean,
        Kind::String(_) => PrimitiveType::String,
        // "integer" which matches any number with a zero fractional part
        // https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.1
        Kind::Number(number) if Decimal::from(number).is_integer() => PrimitiveType::Integer,
        Kind::Number(_) => PrimitiveType::Number,
        Kind::Array(_) => PrimitiveType::Array,
        Kind::Object(_) => PrimitiveType::Object,
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

#[cfg(test)]
mod tests {
    use jjpwrgem_parse::{SyntaxOptions, ast::Document};

    use super::*;

    fn doc(json: &str) -> Document<&str> {
        Document::parse(json, SyntaxOptions::JSON).unwrap()
    }

    // r[verify json-schema-v7.type.primitive]
    #[rstest::rstest]
    #[case("null", PrimitiveType::Null)]
    #[case("true", PrimitiveType::Boolean)]
    #[case(r#""text""#, PrimitiveType::String)]
    #[case("[1]", PrimitiveType::Array)]
    #[case("{}", PrimitiveType::Object)]
    fn value_maps_to_primitive(#[case] json: &str, #[case] expected: PrimitiveType) {
        let doc = doc(json);
        assert_eq!(value_as_primitive(doc.root()), expected);
    }

    // r[verify json-schema-v7.type.integer-subset]
    #[rstest::rstest]
    #[case("123", PrimitiveType::Integer)]
    #[case("123.0", PrimitiveType::Integer)]
    #[case("1e3", PrimitiveType::Integer)]
    #[case("1.5", PrimitiveType::Number)]
    #[case("1.5e1", PrimitiveType::Integer)]
    #[case("1e-1", PrimitiveType::Number)]
    fn number_is_classified_by_value(#[case] json: &str, #[case] expected: PrimitiveType) {
        let doc = doc(json);
        assert_eq!(value_as_primitive(doc.root()), expected);
    }

    // r[verify json-schema-v7.type.integer-subset]
    #[rstest::rstest]
    #[case(PrimitiveType::Integer, PrimitiveType::Number, true)]
    #[case(PrimitiveType::Number, PrimitiveType::Integer, false)]
    #[case(PrimitiveType::Integer, PrimitiveType::Integer, true)]
    #[case(PrimitiveType::String, PrimitiveType::Number, false)]
    fn is_subset_of(#[case] lhs: PrimitiveType, #[case] rhs: PrimitiveType, #[case] valid: bool) {
        assert_eq!(lhs.is_subset(rhs), valid);
    }
}
