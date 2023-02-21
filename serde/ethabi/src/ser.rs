use regex::Regex;
use serde::{ser, Serialize};

use thiserror::Error;

/// followed by the minimum number of zero-bytes such that `len(bytes)` is a multiple of 32
fn padding_right(mut bytes: Vec<u8>) -> Vec<u8> {
    let padding_zeros = 32 - bytes.len() % 32;

    if padding_zeros == 32 {
        return bytes;
    } else {
        let mut padding = vec![0u8; padding_zeros];

        bytes.append(&mut padding);

        bytes
    }
}

/// Abi serializer error variant
#[derive(Debug, Error)]
pub enum AbiSerError {
    #[error("Unknown error,{0}")]
    Unknown(String),
    #[error("Before encoding a new element, you need to close the bytes group first.")]
    UnclosedBytes,

    #[error("Try to encoding mulit-elements when root type is not a tuple")]
    RootIsNotTuple,
    #[error("Try to close tuple/bytes without calling start_tuple/start_bytes first.")]
    NotFoundGroup,
    #[error("Try to close serializer without calling end_group first.")]
    UnclosedGroup,
    #[error("Call start_bytes first")]
    OpenBytesFirst,
    #[error("Unsupport serialize type, {0}")]
    UnsupportType(String),
    #[error("Input bytes<M> data out of range, 0 < M < 32")]
    Byte32OutOfRange,
}

impl ser::Error for AbiSerError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Unknown(msg.to_string())
    }
}

#[derive(Debug)]
enum AbiElement {
    Dynamic(Vec<u8>),
    Static([u8; 32]),
    Byte(u8),
}

#[derive(Debug, Default)]
struct AbiTupleSerializer {
    /// True if this tuple is variable length types, e.g: T[] or bytes
    variable_length: bool,
    /// Tuple's variably sized elements
    elements: Vec<AbiElement>,
}

impl AbiTupleSerializer {
    fn append_bytes(&mut self, bytes: Vec<u8>) {
        self.elements.push(AbiElement::Dynamic(bytes));
    }

    fn append_bytes32(&mut self, bytes32: [u8; 32]) {
        self.elements.push(AbiElement::Static(bytes32));
    }

    fn append_byte(&mut self, byte: u8) {
        self.elements.push(AbiElement::Byte(byte))
    }

    fn finalize(mut self) -> Result<Vec<u8>, AbiSerError> {
        // let len = self.elements.len();
        // let variable_length = self.variable_length;

        let offset = self.elements.len() * 32;

        let mut headers = if self.variable_length {
            let mut encoder = AbiSerializer::default();

            encoder.encode_usize(self.elements.len())?;

            encoder.finalize()?
        } else {
            vec![]
        };

        let mut tails = vec![];

        for (_index, data) in self.elements.iter_mut().enumerate() {
            match data {
                AbiElement::Dynamic(v) => {
                    let current_offset = offset + tails.len();

                    let mut encoder = AbiSerializer::default();

                    encoder.encode_usize(current_offset)?;

                    headers.append(&mut encoder.finalize()?);

                    tails.append(v);
                }
                AbiElement::Static(v) => {
                    headers.append(&mut v.to_vec());
                }
                AbiElement::Byte(b) => {
                    let mut buff = [0; 32];

                    buff[31] = *b;

                    headers.append(&mut buff.to_vec());
                }
            }
        }

        headers.append(&mut tails);

        Ok(headers)
    }
}

/// Abi format serializer for [`serde`](https://serde.rs/)
#[derive(Debug, Default)]
pub struct AbiSerializer {
    buff: Vec<u8>,
    groups: Vec<AbiTupleSerializer>,
}

impl AbiSerializer {
    pub fn start_tuple(&mut self, variable_length: bool) -> Result<(), AbiSerError> {
        if !self.buff.is_empty() {
            return Err(AbiSerError::RootIsNotTuple);
        }

        self.groups.push(AbiTupleSerializer {
            variable_length,
            ..Default::default()
        });

        Ok(())
    }

    pub fn end_tuple(&mut self) -> Result<(), AbiSerError> {
        let mut buff = match self.groups.pop() {
            Some(tuple) => tuple.finalize()?,
            None => {
                return Err(AbiSerError::NotFoundGroup);
            }
        };

        match self.groups.last_mut() {
            Some(tuple) => {
                assert!(buff.len() >= 32);

                tuple.append_bytes(buff);
            }
            _ => {
                if !self.buff.is_empty() {
                    return Err(AbiSerError::RootIsNotTuple);
                }

                self.buff.append(&mut buff);
            }
        }

        Ok(())
    }

    /// of length k (which is assumed to be of type uint256):
    /// enc(X) = enc(k) pad_right(X), i.e. the number of bytes i
    /// s encoded as a uint256 followed by the actual value of X
    /// as a byte sequence, followed by the minimum number of
    /// zero-bytes such that len(enc(X)) is a multiple of 32.
    pub fn encode_bytes(&mut self, bytes: Vec<u8>) -> Result<(), AbiSerError> {
        let mut serializer = AbiSerializer::default();

        serializer.encode_usize(bytes.len())?;

        let mut with_len = serializer.finalize()?;

        let mut bytes = padding_right(bytes);

        with_len.append(&mut bytes);

        match self.groups.last_mut() {
            Some(tuple) => {
                tuple.append_bytes(with_len);
            }
            // Some(AbiGroupSerializer::Bytes(_)) => return Err(AbiSerError::UnclosedBytes),
            None => {
                self.buff = with_len;
            }
        }

        Ok(())
    }

    /// Encode rust `usize` to `contract unit`
    pub fn encode_usize(&mut self, value: usize) -> Result<(), AbiSerError> {
        let bytes = value.to_be_bytes();

        self.encode_bytes32(&bytes)
    }

    pub fn encode_bytes32(&mut self, bytes: &[u8]) -> Result<(), AbiSerError> {
        if bytes.len() > 32 {
            return Err(AbiSerError::Byte32OutOfRange);
        }

        let mut buff = [0; 32];

        buff[(32 - bytes.len())..].clone_from_slice(bytes);

        match self.groups.last_mut() {
            Some(tuple) => {
                tuple.append_bytes32(buff);
            }
            None => {
                self.buff = buff.to_vec();
            }
        }

        Ok(())
    }

    /// Append byte to openned group bytes
    pub fn append_byte(&mut self, byte: u8) -> anyhow::Result<(), AbiSerError> {
        match self.groups.last_mut() {
            Some(tuple) => {
                tuple.append_byte(byte);
                Ok(())
            }
            None => {
                return Err(AbiSerError::OpenBytesFirst);
            }
        }
    }

    /// Close serializer and returns abi data
    pub fn finalize(self) -> Result<Vec<u8>, AbiSerError> {
        if !self.groups.is_empty() {
            return Err(AbiSerError::NotFoundGroup);
        }

        Ok(self.buff)
    }
}

impl<'a> ser::Serializer for &'a mut AbiSerializer {
    type Ok = ();
    type Error = AbiSerError;

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
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if v {
            self.encode_usize(1)
        } else {
            self.encode_usize(0)
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.encode_bytes(v.to_vec())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!("Contract abi don't support rust char")
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(AbiSerError::UnsupportType("f64".to_owned()))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_be_bytes();

        let mut buff = [0u8; 32];

        buff[(32 - bytes.len())..].copy_from_slice(&bytes);

        self.encode_bytes32(&bytes)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i128(v as i128)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i128(v as i128)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_i128(v as i128)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i128(v as i128)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(AbiSerError::UnsupportType("map".to_owned()))
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

                self.encode_bytes(bytes.to_owned())
            }
            "address" => {
                let bytes = unsafe { (value as *const T).cast::<[u8; 32]>().as_ref().unwrap() };

                return self.encode_bytes32(bytes);
            }
            _ => {
                let bytes_regex = Regex::new(r"^bytes(\d{1,2})$").unwrap();
                let int_regex = Regex::new(r"^(u)?int(\d{1,3})$").unwrap();

                if let Some(caps) = bytes_regex.captures(name) {
                    let len: usize = caps[1].parse().unwrap();
                    if len <= 32 {
                        let bytes =
                            unsafe { (value as *const T).cast::<[u8; 32]>().as_ref().unwrap() };

                        return self.encode_bytes32(bytes);
                    }
                }

                if let Some(caps) = int_regex.captures(name) {
                    let len: usize = caps[2].parse().unwrap();
                    if len <= 256 {
                        let bytes =
                            unsafe { (value as *const T).cast::<[u8; 32]>().as_ref().unwrap() };

                        return self.encode_bytes32(bytes);
                    }
                }

                return Err(AbiSerError::UnsupportType(name.to_owned()));
            }
        }
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!("Contract abi don't support rust enum")
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!("Contract abi don't support rust Option")
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.start_tuple(true)?;

        Ok(self)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!("Contract abi don't support rust Option")
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.encode_bytes(v.as_bytes().to_vec())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unimplemented!("Contract abi don't support rust enum")
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.start_tuple(false)?;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.start_tuple(false)?;
        Ok(self)
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
        let bytes = v.to_be_bytes();

        let mut buff = [0xffu8; 32];

        buff[(32 - bytes.len())..].copy_from_slice(&bytes);

        self.encode_bytes32(&bytes)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u128(v as u128)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u128(v as u128)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_u128(v as u128)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u128(v as u128)
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

impl<'a> ser::SerializeMap for &'a mut AbiSerializer {
    type Error = AbiSerError;

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

impl<'a> ser::SerializeSeq for &'a mut AbiSerializer {
    type Error = AbiSerError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_tuple()?;

        Ok(())
    }

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;

        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut AbiSerializer {
    type Error = AbiSerError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_tuple()?;

        Ok(())
    }

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;

        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut AbiSerializer {
    type Error = AbiSerError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_tuple()?;

        Ok(())
    }

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;

        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut AbiSerializer {
    type Error = AbiSerError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_tuple()?;

        Ok(())
    }

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;

        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut AbiSerializer {
    type Error = AbiSerError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_tuple()?;

        Ok(())
    }

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;

        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut AbiSerializer {
    type Error = AbiSerError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_tuple()?;

        Ok(())
    }

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;

        Ok(())
    }
}

/// Serialize rust value to contract abi format.
pub fn to_abi<S: Serialize>(value: &S) -> anyhow::Result<Vec<u8>> {
    let mut serializer = AbiSerializer::default();

    value.serialize(&mut serializer).expect("");

    Ok(serializer.finalize()?)
}
