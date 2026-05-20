#![expect(
    clippy::unnecessary_wraps,
    reason = "serde trait methods require Result<(), Error>"
)]

use std::fmt::{Display, Write as _};

use serde::ser::{
    self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};

use super::error::Error;
use crate::{
    format::{Emitter, uglify::UglifyEmitVisitor},
    tokens::{NULL, lexical::JsonChar},
};

fn emit_json_string(emitter: &mut impl Emitter, s: &str) {
    emitter.push('"');

    let mut start = 0;
    for (idx, ch) in s.char_indices() {
        let json_char = JsonChar(ch);
        let escaped = match json_char.minimal_escape() {
            Some(escaped) => Some(escaped),
            None if json_char.is_control() => {
                emitter.push_str(&s[start..idx]);
                emitter.push_str(&json_char.escape());
                start = idx + ch.len_utf8();
                continue;
            }
            None => None,
        };

        if let Some(escaped) = escaped {
            emitter.push_str(&s[start..idx]);
            emitter.push_str(escaped);
            start = idx + ch.len_utf8();
        }
    }

    emitter.push_str(&s[start..]);
    emitter.push('"');
}

impl UglifyEmitVisitor {
    fn serialize_number<N: Display>(&mut self, number: N) -> Result<(), Error> {
        write!(&mut self.buf, "{number}").expect("write to string");
        Ok(())
    }
}

fn emit_item_prefix(first: &mut bool, emitter: &mut impl Emitter) {
    if !*first {
        emitter.emit_item_delim();
    }
    *first = false;
}

fn map_key_bytes_error() -> Error {
    ser::Error::custom("JSON object keys cannot be byte arrays")
}

fn map_key_enum_error() -> Error {
    ser::Error::custom("JSON object keys cannot be enum objects")
}

fn map_key_scalar_error() -> Error {
    ser::Error::custom("JSON object keys must be scalars")
}

macro_rules! serialize_number_methods {
    ($($name:ident : $ty:ty),* $(,)?) => {
        $(
            fn $name(self, v: $ty) -> Result<Self::Ok, Self::Error> {
                self.serialize_number(v)
            }
        )*
    };
}

macro_rules! impl_seq_like {
    ($trait_name:ident, $method_name:ident) => {
        impl $trait_name for SeqSerializer<'_> {
            type Ok = ();
            type Error = Error;

            fn $method_name<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + serde::Serialize,
            {
                self.serialize_item(value)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                self.end_array()
            }
        }
    };
}

macro_rules! map_key_scalar_to_string {
    ($($name:ident : $ty:ty),* $(,)?) => {
        $(
            fn $name(self, v: $ty) -> Result<Self::Ok, Self::Error> {
                Ok(v.to_string())
            }
        )*
    };
}

macro_rules! impossible_map_key_methods {
    ($($name:ident ( $($arg:ident : $arg_ty:ty),* ) -> $ret:ident => $err:ident;)* $(,)?) => {
        $(
            fn $name(self, $($arg: $arg_ty),*) -> Result<Self::$ret, Self::Error> {
                Err($err())
            }
        )*
    };
}

impl<'a> ser::Serializer for &'a mut UglifyEmitVisitor {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = SeqSerializer<'a>;
    type SerializeTupleStruct = SeqSerializer<'a>;
    type SerializeTupleVariant = TupleVariantSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = MapSerializer<'a>;
    type SerializeStructVariant = StructVariantSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.emit_boolean(v);
        Ok(())
    }

    serialize_number_methods!(
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_u128: u128,
    );

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        if v.is_finite() {
            self.serialize_number(v)
        } else {
            self.serialize_unit()
        }
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        if v.is_finite() {
            self.serialize_number(v)
        } else {
            self.serialize_unit()
        }
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut tmp = [0; 4];
        self.serialize_str(v.encode_utf8(&mut tmp))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        emit_json_string(self, v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            SerializeSeq::serialize_element(&mut seq, byte)?;
        }
        SerializeSeq::end(seq)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.emit_null();
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.emit_object_open();
        emit_json_string(self, variant);
        self.emit_key_val_delim();
        value.serialize(&mut *self)?;
        self.emit_object_close();
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.emit_array_open();
        Ok(SeqSerializer {
            serializer: self,
            first: true,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.emit_object_open();
        emit_json_string(self, variant);
        self.emit_key_val_delim();
        self.emit_array_open();
        Ok(TupleVariantSerializer {
            serializer: self,
            first: true,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.emit_object_open();
        Ok(MapSerializer {
            serializer: self,
            first: true,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.emit_object_open();
        emit_json_string(self, variant);
        self.emit_key_val_delim();
        self.emit_object_open();
        Ok(StructVariantSerializer {
            serializer: self,
            first: true,
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&value.to_string())
    }
}

#[derive(Debug)]
pub struct SeqSerializer<'a> {
    serializer: &'a mut UglifyEmitVisitor,
    first: bool,
}

impl SeqSerializer<'_> {
    fn serialize_item<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + serde::Serialize,
    {
        emit_item_prefix(&mut self.first, self.serializer);
        value.serialize(&mut *self.serializer)
    }

    fn end_array(self) -> Result<(), Error> {
        self.serializer.emit_array_close();
        Ok(())
    }
}

impl_seq_like!(SerializeSeq, serialize_element);
impl_seq_like!(SerializeTuple, serialize_element);
impl_seq_like!(SerializeTupleStruct, serialize_field);

#[derive(Debug)]
pub struct TupleVariantSerializer<'a> {
    serializer: &'a mut UglifyEmitVisitor,
    first: bool,
}

impl SerializeTupleVariant for TupleVariantSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        emit_item_prefix(&mut self.first, self.serializer);
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.emit_array_close();
        self.serializer.emit_object_close();
        Ok(())
    }
}

#[derive(Debug)]
pub struct MapSerializer<'a> {
    serializer: &'a mut UglifyEmitVisitor,
    first: bool,
}

impl MapSerializer<'_> {
    fn serialize_key_str(&mut self, key: &str) {
        emit_item_prefix(&mut self.first, self.serializer);
        emit_json_string(self.serializer, key);
        self.serializer.emit_key_val_delim();
    }

    fn end_object(self) -> Result<(), Error> {
        self.serializer.emit_object_close();
        Ok(())
    }
}

impl SerializeMap for MapSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let key = key.serialize(MapKeySerializer)?;
        self.serialize_key_str(&key);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_object()
    }
}

impl SerializeStruct for MapSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.serialize_key_str(key);
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_object()
    }
}

#[derive(Debug)]
pub struct StructVariantSerializer<'a> {
    serializer: &'a mut UglifyEmitVisitor,
    first: bool,
}

impl SerializeStructVariant for StructVariantSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        emit_item_prefix(&mut self.first, self.serializer);
        emit_json_string(self.serializer, key);
        self.serializer.emit_key_val_delim();
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.emit_object_close();
        self.serializer.emit_object_close();
        Ok(())
    }
}

struct MapKeySerializer;

impl ser::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = ser::Impossible<String, Error>;
    type SerializeTuple = ser::Impossible<String, Error>;
    type SerializeTupleStruct = ser::Impossible<String, Error>;
    type SerializeTupleVariant = ser::Impossible<String, Error>;
    type SerializeMap = ser::Impossible<String, Error>;
    type SerializeStruct = ser::Impossible<String, Error>;
    type SerializeStructVariant = ser::Impossible<String, Error>;

    map_key_scalar_to_string!(
        serialize_bool: bool,
        serialize_i8: i8,
        serialize_i16: i16,
        serialize_i32: i32,
        serialize_i64: i64,
        serialize_i128: i128,
        serialize_u8: u8,
        serialize_u16: u16,
        serialize_u32: u32,
        serialize_u64: u64,
        serialize_u128: u128,
        serialize_f32: f32,
        serialize_f64: f64,
        serialize_char: char,
    );

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_owned())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(NULL.to_owned())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(NULL.to_owned())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.to_owned())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(map_key_enum_error())
    }

    impossible_map_key_methods! {
        serialize_bytes(_v: &[u8]) -> Ok => map_key_bytes_error;
        serialize_seq(_len: Option<usize>) -> SerializeSeq => map_key_scalar_error;
        serialize_tuple(_len: usize) -> SerializeTuple => map_key_scalar_error;
        serialize_tuple_struct(_name: &'static str, _len: usize) -> SerializeTupleStruct => map_key_scalar_error;
        serialize_tuple_variant(_name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> SerializeTupleVariant => map_key_scalar_error;
        serialize_map(_len: Option<usize>) -> SerializeMap => map_key_scalar_error;
        serialize_struct(_name: &'static str, _len: usize) -> SerializeStruct => map_key_scalar_error;
        serialize_struct_variant(_name: &'static str, _variant_index: u32, _variant: &'static str, _len: usize) -> SerializeStructVariant => map_key_scalar_error;
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde::Serialize;

    use crate::format::serde::uglify_serializable;

    #[derive(Serialize)]
    struct Example<'a> {
        text: &'a str,
        values: [u8; 3],
        flag: bool,
    }

    #[test]
    fn uglify_serializable_writes_expected_json() {
        let actual = uglify_serializable(Example {
            text: "quote=\" slash=/ newline=\n tab=\t",
            values: [1, 2, 3],
            flag: true,
        })
        .unwrap();

        assert_eq!(
            actual,
            r#"{"text":"quote=\" slash=/ newline=\n tab=\t","values":[1,2,3],"flag":true}"#
        );
    }

    #[test]
    fn uglify_serializable_supports_externally_tagged_variants() {
        assert_eq!(
            uglify_serializable(Result::<u8, &str>::Ok(7)).unwrap(),
            r#"{"Ok":7}"#
        );
        assert_eq!(
            uglify_serializable(Result::<u8, &str>::Err("x")).unwrap(),
            r#"{"Err":"x"}"#
        );
    }

    #[test]
    fn uglify_serializable_stringifies_scalar_map_keys() {
        let actual = uglify_serializable(BTreeMap::from([(12, false), (34, true)])).unwrap();
        assert_eq!(actual, r#"{"12":false,"34":true}"#);
    }
}
