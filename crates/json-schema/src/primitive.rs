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
