//! EIP712 types serializer

use std::collections::HashMap;

use regex::Regex;
use serde::{ser, Deserialize, Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum TypeDefinitionError {
    #[error("{0}")]
    Unknown(String),
    #[error("Invalid primary type, implementation only support structure.")]
    InvalidPrimary,
    #[error("Call end_tuple before calling start_tuple.")]
    StartTupleFirst,
    #[error("append_field_name and append_field_type must be called in pairs")]
    FiledNameTypePair,

    #[error("Unsupport type for eip712, {0}")]
    UnsupportType(String),

    #[error("start_tuple and end_tuple must be called in pairs")]
    UnclosedTuple,

    #[error("Call append_field_name first")]
    AppendFieldName,
}

impl ser::Error for TypeDefinitionError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Unknown(msg.to_string())
    }
}

#[derive(Debug, Default)]
struct TupleEncoder {
    name: String,
    field_names: Vec<String>,
    field_types: Vec<String>,
}

impl TupleEncoder {
    fn append_field_name(&mut self, name: &str) -> Result<(), TypeDefinitionError> {
        if self.field_names.len() != self.field_types.len() {
            return Err(TypeDefinitionError::FiledNameTypePair);
        }
        self.field_names.push(name.to_owned());

        Ok(())
    }

    fn append_field_type(&mut self, type_name: &str) -> Result<(), TypeDefinitionError> {
        if self.field_names.len() != self.field_types.len() + 1 {
            return Err(TypeDefinitionError::AppendFieldName);
        }

        self.field_types.push(type_name.to_owned());

        Ok(())
    }

    fn finalize(self) -> Result<Vec<TypeDefinitionField>, TypeDefinitionError> {
        if self.field_names.len() != self.field_types.len() {
            return Err(TypeDefinitionError::FiledNameTypePair);
        }

        let mut fields = vec![];

        for (index, field) in self.field_names.into_iter().enumerate() {
            fields.push(TypeDefinitionField {
                name: field,
                r#type: self.field_types[index].to_owned(),
            });
        }

        Ok(fields)
    }

    fn pop_field(&mut self) {
        self.field_names
            .pop()
            .ok_or(TypeDefinitionError::FiledNameTypePair)
            .unwrap();
    }
}

#[derive(Debug, Default)]
pub struct EIP712TypeDefinition {
    primary_type: String,
    handled: HashMap<String, TypeDefinition>,
    tuples: Vec<TupleEncoder>,
}

impl EIP712TypeDefinition {
    /// Returns json string for types, and close serializer.
    pub fn finalize(self) -> Result<HashMap<String, TypeDefinition>, TypeDefinitionError> {
        if !self.tuples.is_empty() {
            return Err(TypeDefinitionError::UnclosedTuple);
        }

        Ok(self.handled)
    }

    /// Append one field name of structure.
    /// [`append_field_name`](EIP712TypeDefinition::append_field_name) and
    /// [`append_field_type`](EIP712TypeDefinition::append_field_type) must be called sequentially.
    pub fn append_field_name(&mut self, name: &str) -> Result<(), TypeDefinitionError> {
        if let Some(tuple) = self.tuples.last_mut() {
            tuple.append_field_name(name)?;
            Ok(())
        } else {
            return Err(TypeDefinitionError::InvalidPrimary);
        }
    }
    /// Call this function only when serializing none
    fn pop_field(&mut self) -> Result<(), TypeDefinitionError> {
        if let Some(tuple) = self.tuples.last_mut() {
            tuple.pop_field();
            Ok(())
        } else {
            return Err(TypeDefinitionError::InvalidPrimary);
        }
    }

    /// Append one field type name of parent structure.
    /// [`append_field_name`](EIP712TypeDefinition::append_field_name) and
    /// [`append_field_type`](EIP712TypeDefinition::append_field_type) must be sequentially.
    pub fn append_field_type(&mut self, type_name: &str) -> Result<(), TypeDefinitionError> {
        if let Some(tuple) = self.tuples.last_mut() {
            tuple.append_field_type(type_name)?;
            Ok(())
        } else {
            return Err(TypeDefinitionError::InvalidPrimary);
        }
    }

    /// Start serialize one new tuple, if returns true.
    /// If returns false, no further serialization of this tuple is required.
    pub fn start_tuple(&mut self, name: &str) -> Result<bool, TypeDefinitionError> {
        if self.handled.contains_key(name) {
            // append field type directly.
            if let Some(tuple) = self.tuples.last_mut() {
                tuple.append_field_type(name)?;
            } else {
                return Err(TypeDefinitionError::InvalidPrimary);
            }

            return Ok(false);
        }

        self.handled.insert(name.to_owned(), vec![]);

        // push new tuple serializer into the processing stack.
        self.tuples.push(TupleEncoder {
            name: name.to_owned(),
            ..Default::default()
        });

        Ok(true)
    }

    /// Finish one tuple serialize.
    pub fn end_tuple(&mut self) -> Result<(), TypeDefinitionError> {
        if let Some(tuple) = self.tuples.pop() {
            let name = tuple.name.clone();
            let type_declare = tuple.finalize()?;

            if let Some(parent) = self.tuples.last_mut() {
                parent.append_field_type(&name)?;
            } else {
                // This tuple is primary type.
                if self.tuples.is_empty() {
                    self.primary_type = name.to_owned();
                }
            }

            self.handled.insert(name, type_declare);
        } else {
            return Err(TypeDefinitionError::StartTupleFirst);
        }
        Ok(())
    }
}

impl<'a> Serializer for &'a mut EIP712TypeDefinition {
    type Ok = ();
    type Error = TypeDefinitionError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = EIP712TypeDefinitionSerializeStruct<'a>;
    type SerializeStructVariant = Self;

    fn is_human_readable(&self) -> bool {
        false
    }
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!("Contract abi don't support rust char")
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        unimplemented!("EIP712 don't support f32")
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        unimplemented!("EIP712 don't support f64")
    }

    fn serialize_i128(self, _v: i128) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("int128")
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("int16")
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("int32")
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("int16")
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("int8")
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!("EIP712 don't support map")
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        match name {
            "bytes" => self.append_field_type("bytes"),
            "address" => self.append_field_type("address"),
            _ => {
                let bytes_regex = Regex::new(r"^bytes(\d{1,2})$").unwrap();
                let int_regex = Regex::new(r"^(u)?int(\d{1,3})$").unwrap();

                if let Some(caps) = bytes_regex.captures(name) {
                    let len: usize = caps[1].parse().unwrap();
                    if len <= 32 {
                        return self.append_field_type(name);
                    }
                }

                if let Some(caps) = int_regex.captures(name) {
                    let len: usize = caps[2].parse().unwrap();
                    if len <= 256 {
                        return self.append_field_type(name);
                    }
                }

                return Err(TypeDefinitionError::UnsupportType(name.to_owned()));
            }
        }
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        return Err(TypeDefinitionError::UnsupportType(format!(
            "enum {}::{}",
            name, variant
        )));
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.pop_field()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unimplemented!("EIP712 don't support rust enum")
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("string")
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        let need_serialize = self.start_tuple(name)?;

        Ok(EIP712TypeDefinitionSerializeStruct {
            need_serialize,
            serializer: self,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unimplemented!("EIP712 don't support rust enum")
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unimplemented!("EIP712 don't support serialize tuple")
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unimplemented!("EIP712 don't support serialize tuple_struct")
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unimplemented!("Contract abi don't support rust enum")
    }

    fn serialize_u128(self, _v: u128) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("uint128")
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("uint16")
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("uint32")
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("uint64")
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        self.append_field_type("uint8")
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!("Contract abi don't support rust enum")
    }
}

impl<'a> ser::SerializeMap for &'a mut EIP712TypeDefinition {
    type Error = TypeDefinitionError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }
}

impl<'a> ser::SerializeSeq for &'a mut EIP712TypeDefinition {
    type Error = TypeDefinitionError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }
}

pub struct EIP712TypeDefinitionSerializeStruct<'a> {
    need_serialize: bool,
    serializer: &'a mut EIP712TypeDefinition,
}

impl<'a> ser::SerializeStruct for EIP712TypeDefinitionSerializeStruct<'a> {
    type Error = TypeDefinitionError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.need_serialize {
            self.serializer.end_tuple()?;
        }

        Ok(())
    }

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        if self.need_serialize {
            self.serializer.append_field_name(key)?;
            value.serialize(&mut *self.serializer)?;
        }

        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut EIP712TypeDefinition {
    type Error = TypeDefinitionError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTuple for &'a mut EIP712TypeDefinition {
    type Error = TypeDefinitionError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut EIP712TypeDefinition {
    type Error = TypeDefinitionError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut EIP712TypeDefinition {
    type Error = TypeDefinitionError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!()
    }
}

/// EIP712 type definition structure.
pub type TypeDefinition = Vec<TypeDefinitionField>;

/// EIP712 field of type definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeDefinitionField {
    pub name: String,
    pub r#type: String,
}

/// Convert [`Serialize`] value to eip712 encode type.
pub fn eip712_type_definitions<S: Serialize>(
    value: &S,
) -> Result<HashMap<String, TypeDefinition>, TypeDefinitionError> {
    let mut serializer = EIP712TypeDefinition::default();

    value.serialize(&mut serializer)?;

    serializer.finalize()
}
