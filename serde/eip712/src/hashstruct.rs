//! EIP712 `encodeData` implementation using the serde [`Serialize`] framework.

use std::collections::HashMap;

use regex::Regex;
use serde::{ser, Serialize, Serializer};
use sha3::{Digest, Keccak256};

use crate::TypeDefinition;

#[derive(Debug, thiserror::Error)]
pub enum EncodeDataError {
    #[error("{0}")]
    Unknown(String),

    #[error("Unsupport type for eip712, {0}")]
    UnsupportType(String),

    #[error("Type definition not found, {0}")]
    TypeDefinitionNotFound(String),

    #[error("Close tuple before calling start_tuple function")]
    EndTuple,
    #[error("Call start_tuple first, eip712 root type must be a structure")]
    StartTuple,
    #[error("start_tuple and end_tuple must be called in pairs")]
    UnclosedTuple,
    #[error("Encode data is empty")]
    Empty,
}

impl ser::Error for EncodeDataError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Unknown(msg.to_string())
    }
}

#[derive(Debug, Default)]
struct TupleEncoder {
    names: Vec<String>,
    fields: Vec<[u8; 32]>,
}

impl TupleEncoder {
    fn append_element(&mut self, data: [u8; 32]) {
        self.fields.push(data);
    }

    fn append_element_name(&mut self, name: &str) {
        self.names.push(name.to_owned());
    }

    fn finalize(self, type_hash: [u8; 32], definition: TypeDefinition) -> [u8; 32] {
        let mut hasher = Keccak256::new();

        hasher.update(&type_hash);

        for field in definition {
            let index = self
                .names
                .iter()
                .enumerate()
                .find(|(_, n)| n.as_str() == field.name)
                .map(|(index, _)| index)
                .unwrap();

            hasher.update(&self.fields[index])
        }

        hasher.finalize().into()
    }
}

#[derive(Debug)]
pub struct EIP712StructHasher<'a> {
    hashed: Option<[u8; 32]>,
    tuple_stack: Vec<TupleEncoder>,
    types: &'a HashMap<String, TypeDefinition>,
    primary_type: &'a str,
    type_stack: Vec<(String, TypeDefinition)>,
}

fn extract_type_name(name: &str) -> String {
    let regex = Regex::new(r#"^([^\[]+)\[\d*\]$"#).unwrap();

    if let Some(caps) = regex.captures(name) {
        caps[1].to_owned()
    } else {
        name.to_owned()
    }
}

impl<'a> EIP712StructHasher<'a> {
    /// Returns json string for types, and close serializer.
    pub fn finalize(mut self) -> Result<[u8; 32], EncodeDataError> {
        if !self.tuple_stack.is_empty() {
            return Err(EncodeDataError::UnclosedTuple);
        }

        if let Some(hashed) = self.hashed.take() {
            Ok(hashed)
        } else {
            return Err(EncodeDataError::Empty);
        }
    }

    fn start_encode_type(&mut self, name: &str) -> Result<(), EncodeDataError> {
        self.append_element_name(name)?;

        if self.type_stack.is_empty() {
            if let Some(definition) = self.types.get(self.primary_type) {
                self.type_stack
                    .push((self.primary_type.to_owned(), definition.to_owned()));
            } else {
                return Err(EncodeDataError::TypeDefinitionNotFound(
                    self.primary_type.to_owned(),
                ));
            }
        }

        let parent_type = self.type_stack.last().unwrap();

        if let Some(type_definition) = parent_type.1.iter().find(|d| d.name == name) {
            log::debug!("start encode type {}", type_definition.r#type);

            let type_name = extract_type_name(&type_definition.r#type);

            if let Some(definition) = self.types.get(&type_name) {
                self.type_stack.push((type_name, definition.to_owned()));

                return Ok(());
            } else {
                // assume is builtin type
                log::debug!("process builtin type {}", type_name);
                return Ok(());
            }
        } else {
            log::debug!("maybe none field {}", name);
            return Ok(());
            // return Err(EncodeDataError::TypeDefinitionNotFound(name.to_owned()));
        }
    }

    // pub fn end_encode_type(&mut self) -> Result<(), EncodeDataError> {
    //     self.type_stack.pop();

    //     Ok(())
    // }

    fn type_hash(&mut self) -> Result<[u8; 32], EncodeDataError> {
        if self.type_stack.is_empty() {
            if let Some(definition) = self.types.get(self.primary_type) {
                self.type_stack
                    .push((self.primary_type.to_owned(), definition.to_owned()));
            } else {
                return Err(EncodeDataError::TypeDefinitionNotFound(
                    self.primary_type.to_owned(),
                ));
            }
        }

        let primary_type = self.type_stack.last().unwrap();

        let mut types = HashMap::<String, (String, TypeDefinition)>::new();

        let mut stack = vec![primary_type];

        while !stack.is_empty() {
            let current = stack.pop().unwrap();
            for field in &current.1 {
                let type_name = extract_type_name(&field.r#type);

                if !types.contains_key(&type_name) {
                    if let Some(definition) = self.types.get(&type_name) {
                        types.insert(type_name.clone(), (type_name, definition.to_owned()));
                    }
                }
            }
        }

        let mut keys = types.keys().collect::<Vec<_>>();

        keys.sort_by(|a, b| a.cmp(b));

        let mut sorted = vec![primary_type];

        for key in keys {
            let value = types.get(key.as_str());
            sorted.push(value.unwrap());
        }

        let encode_type = sorted
            .iter()
            .map(|(name, fields)| {
                let fields = fields
                    .iter()
                    .map(|f| format!("{} {}", f.r#type, f.name))
                    .collect::<Vec<_>>();
                format!("{}({})", name, fields.join(","))
            })
            .collect::<Vec<_>>()
            .join("");

        log::debug!("encode type {}", encode_type);

        Ok(Keccak256::new()
            .chain_update(encode_type.as_bytes())
            .finalize()
            .into())
    }

    /// Start encode tuple(e.g, <Type>[5], Structure)
    pub fn start_tuple(&mut self) -> Result<(), EncodeDataError> {
        self.tuple_stack.push(TupleEncoder {
            ..Default::default()
        });

        Ok(())
    }

    pub fn end_tuple(&mut self) -> Result<(), EncodeDataError> {
        if let Some(tuple) = self.tuple_stack.pop() {
            let encode_data = tuple.finalize(self.type_hash()?, self.type_stack.pop().unwrap().1);

            if let Some(tuple) = self.tuple_stack.last_mut() {
                tuple.append_element(encode_data);
            } else {
                self.hashed = Some(encode_data);
            }

            Ok(())
        } else {
            Err(EncodeDataError::EndTuple)
        }
    }

    pub fn append_element(&mut self, data: [u8; 32]) -> Result<(), EncodeDataError> {
        if let Some(tuple) = self.tuple_stack.last_mut() {
            tuple.append_element(data);
            Ok(())
        } else {
            Err(EncodeDataError::EndTuple)
        }
    }

    fn append_element_name(&mut self, name: &str) -> Result<(), EncodeDataError> {
        if let Some(tuple) = self.tuple_stack.last_mut() {
            tuple.append_element_name(name);
            Ok(())
        } else {
            Err(EncodeDataError::EndTuple)
        }
    }
}

impl<'a, 'b> Serializer for &'a mut EIP712StructHasher<'b> {
    type Ok = ();
    type Error = EncodeDataError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn is_human_readable(&self) -> bool {
        false
    }
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.append_element(Keccak256::new().chain_update(v).finalize().into())
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

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        let mut buff = if v.is_negative() {
            [0u8; 32]
        } else {
            [0xffu8; 32]
        };

        buff[16..].copy_from_slice(&v.to_be_bytes());

        self.append_element(buff)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i128(_v as i128)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i128(_v as i128)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_i128(_v as i128)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i128(_v as i128)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.start_tuple()?;

        Ok(self)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        match name {
            "bytes" => {
                let bytes = unsafe { (value as *const T).cast::<Vec<u8>>().as_ref().unwrap() };

                self.append_element(Keccak256::new().chain_update(bytes).finalize().into())
            }
            "address" => {
                let bytes = unsafe { (value as *const T).cast::<[u8; 32]>().as_ref().unwrap() };

                self.append_element(bytes.to_owned())
            }
            _ => {
                let bytes_regex = Regex::new(r"^bytes(\d{1,2})$").unwrap();
                let int_regex = Regex::new(r"^(u)?int(\d{1,3})$").unwrap();

                if let Some(caps) = bytes_regex.captures(name) {
                    let len: usize = caps[1].parse().unwrap();
                    if len <= 32 {
                        let bytes =
                            unsafe { (value as *const T).cast::<[u8; 32]>().as_ref().unwrap() };

                        return self.append_element(bytes.to_owned());
                    }
                }

                if let Some(caps) = int_regex.captures(name) {
                    let len: usize = caps[2].parse().unwrap();
                    if len <= 256 {
                        let bytes =
                            unsafe { (value as *const T).cast::<[u8; 32]>().as_ref().unwrap() };

                        return self.append_element(bytes.to_owned());
                    }
                }

                return Err(EncodeDataError::UnsupportType(name.to_owned()));
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
        return Err(EncodeDataError::UnsupportType(format!(
            "enum {}::{}",
            name, variant
        )));
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
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

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.append_element(
            Keccak256::new()
                .chain_update(v.as_bytes())
                .finalize()
                .into(),
        )
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.start_tuple()?;

        Ok(self)
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

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        let mut buff = [0u8; 32];

        buff[16..].copy_from_slice(&v.to_be_bytes());

        self.append_element(buff)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u128(_v as u128)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u128(_v as u128)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_u128(_v as u128)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u128(_v as u128)
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

impl<'a, 'b> ser::SerializeStruct for &'a mut EIP712StructHasher<'b> {
    type Error = EncodeDataError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_tuple()
    }

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        self.start_encode_type(key)?;
        value.serialize(&mut **self)
    }
}

impl<'a, 'b> ser::SerializeMap for &'a mut EIP712StructHasher<'b> {
    type Error = EncodeDataError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_tuple()
    }

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let name = serde_json::to_string(key).map_err(ser::Error::custom)?;

        let reg = Regex::new(r#"^"[^"]*"$"#).unwrap();

        if !reg.is_match(&name) {
            return Err(EncodeDataError::Unknown(
                "HashStruct only support map with string key".to_owned(),
            ));
        }

        self.start_encode_type(&name.as_str()[1..(name.len() - 1)])
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)
    }
}

impl<'a, 'b> ser::SerializeSeq for &'a mut EIP712StructHasher<'b> {
    type Error = EncodeDataError;

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

impl<'a, 'b> ser::SerializeStructVariant for &'a mut EIP712StructHasher<'b> {
    type Error = EncodeDataError;

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

impl<'a, 'b> ser::SerializeTuple for &'a mut EIP712StructHasher<'b> {
    type Error = EncodeDataError;

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

impl<'a, 'b> ser::SerializeTupleVariant for &'a mut EIP712StructHasher<'b> {
    type Error = EncodeDataError;

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

impl<'a, 'b> ser::SerializeTupleStruct for &'a mut EIP712StructHasher<'b> {
    type Error = EncodeDataError;

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

/// Calculate struct hash,
/// see [`Definition of hashStruct`](https://eips.ethereum.org/EIPS/eip-712) for more information
pub fn eip712_hash_struct<S: Serialize>(
    primary_type: &str,
    types: &HashMap<String, TypeDefinition>,
    value: &S,
) -> Result<[u8; 32], EncodeDataError> {
    let mut hasher = EIP712StructHasher {
        type_stack: vec![],
        types,
        hashed: None,
        tuple_stack: vec![],
        primary_type,
    };

    value.serialize(&mut hasher)?;

    hasher.finalize()
}
