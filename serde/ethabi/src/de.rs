use bytes::{Buf, Bytes};
use regex::Regex;
use serde::{de, Deserialize};
use thiserror::Error;

/// Abi serializer error variant
#[derive(Debug, Error)]
pub enum AbiDeError {
    #[error("Unknown error,{0}")]
    Unknown(String),
    #[error("Close tuple before calling start_read_tuple,{0}")]
    TupleNotFound(String),

    #[error("Try read next element failed,{0}")]
    InsufficentInputs(String),

    #[error("Next element is static type,{0}")]
    NextIsStatic(String),
}

impl de::Error for AbiDeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Unknown(msg.to_string())
    }
}

#[derive(Debug, Default)]
struct AbiTupleDecoder {
    buff: Bytes,
    offset: usize,
    offset_bytes: usize,
}

impl AbiTupleDecoder {
    fn new(buff: Bytes, dynamic: bool) -> Result<(AbiTupleDecoder, Option<usize>), AbiDeError> {
        if dynamic {
            let mut decoder = AbiDeserializer::new(buff);

            let len: usize = decoder.read_usize()?;

            let remaining = decoder.remaining();

            Ok((
                AbiTupleDecoder {
                    buff: remaining,
                    offset: 0,
                    offset_bytes: 0,
                },
                Some(len),
            ))
        } else {
            Ok((
                AbiTupleDecoder {
                    buff,
                    offset: 0,
                    offset_bytes: 0,
                },
                None,
            ))
        }
    }
    fn start_read_tuple(
        &mut self,
        dynamic: bool,
    ) -> Result<(AbiTupleDecoder, Option<usize>), AbiDeError> {
        let offset_bytes = self.read_header_as_offset_bytes(self.offset)?;

        // Get offset caculated by bytes
        let buff = self.buff.slice(offset_bytes..);

        AbiTupleDecoder::new(buff, dynamic)
    }

    fn end_read_tuple(&mut self, len: usize) -> Result<(), AbiDeError> {
        let offset_bytes = self.read_header_as_offset_bytes(self.offset)?;

        self.offset_bytes = offset_bytes + len;

        self.offset += 1;

        Ok(())
    }

    fn read_bytes(&mut self) -> Result<Bytes, AbiDeError> {
        let offset_bytes = self.read_header_as_offset_bytes(self.offset)?;

        let (bytes, padding_zeros) = read_bytes(self.buff.slice(offset_bytes..))?;

        self.offset += 1;
        self.offset_bytes = offset_bytes + bytes.len() + 32 + padding_zeros;

        Ok(bytes)
    }

    fn read_header(&self, offset: usize) -> Result<Bytes, AbiDeError> {
        let from = (offset) * 32;
        let to = (offset + 1) * 32;

        if from > self.buff.len() - 32 || to > self.buff.len() {
            return Err(
                AbiDeError::InsufficentInputs(format!("Read header({}) content", offset)).into(),
            );
        }

        Ok(self.buff.slice(from..to))
    }

    fn read_header_as_offset_bytes(&self, offset: usize) -> Result<usize, AbiDeError> {
        let mut decoder = AbiDeserializer::new(self.read_header(offset)?);

        decoder.read_usize()
    }

    fn read_one_static(&mut self) -> Result<Bytes, AbiDeError> {
        let buff = self.read_header(self.offset)?;

        self.offset += 1;

        Ok(buff)
    }

    fn remaining(&self) -> Bytes {
        self.buff.slice(self.offset_bytes..)
    }
    /// Comsume self and returns consumed data length
    fn finalize(self) -> usize {
        self.offset_bytes
    }
}

fn read_bytes(buff: Bytes) -> Result<(Bytes, usize), AbiDeError> {
    // Check buff length
    if buff.len() < 32 {
        return Err(AbiDeError::InsufficentInputs(format!("Read bytes length prefix")).into());
    }

    let mut decoder = AbiDeserializer::new(buff.slice(..32));

    let len: usize = decoder.read_usize()?;

    let padding_zeros = 32 - len % 32;

    let end = 32 + len;

    if buff.len() < end {
        return Err(AbiDeError::InsufficentInputs(format!("Read content",)).into());
    }

    Ok((buff.slice(32..end), padding_zeros))
}

/// Abi format decoder
#[derive(Debug, Default)]
pub struct AbiDeserializer {
    root_buff: Bytes,
    tuple_stacks: Vec<AbiTupleDecoder>,
}

impl AbiDeserializer {
    pub fn new<B: Into<Bytes>>(bytes: B) -> Self {
        Self {
            root_buff: bytes.into(),
            tuple_stacks: Default::default(),
        }
    }
    /// Start read a tuple list , returns `length` of list if tuple is a dynamic type.
    pub fn start_read_tuple(&mut self, dynamic: bool) -> Result<Option<usize>, AbiDeError> {
        if let Some(parent) = self.tuple_stacks.last_mut() {
            let (tuple, len) = parent.start_read_tuple(dynamic)?;

            self.tuple_stacks.push(tuple);
            return Ok(len);
        } else {
            let (tuple, len) = AbiTupleDecoder::new(self.root_buff.clone(), dynamic)?;

            self.tuple_stacks.push(tuple);
            return Ok(len);
        }
    }

    /// Stop read one tuple list.
    pub fn end_read_tuple(&mut self) -> Result<(), AbiDeError> {
        if let Some(tuple) = self.tuple_stacks.pop() {
            let len = tuple.finalize();

            if let Some(parent) = self.tuple_stacks.last_mut() {
                parent.end_read_tuple(len)?;
            } else {
                _ = self.root_buff.advance(len);
            }

            Ok(())
        } else {
            return Err(AbiDeError::TupleNotFound("".to_owned()));
        }
    }

    /// Read one static type
    pub fn read_static(&mut self) -> Result<[u8; 32], AbiDeError> {
        if let Some(tuple) = self.tuple_stacks.last_mut() {
            Ok(tuple.read_one_static()?.to_vec().try_into().unwrap())
        } else {
            // ensure decoding buff length is more than 32 bytes
            if self.root_buff.len() < 32 {
                return Err(AbiDeError::InsufficentInputs(
                    "Root buffer length is short than < 32".to_owned(),
                )
                .into());
            }

            let first = self.root_buff.split_to(32);

            // The above line of code, ensures that the conversion must succeed.
            Ok(first.to_vec().try_into().unwrap())
        }
    }

    /// Read bytes like types e.g, string or bytes
    pub fn read_bytes(&mut self) -> Result<Bytes, AbiDeError> {
        if let Some(tuple) = self.tuple_stacks.last_mut() {
            tuple.read_bytes()
        } else {
            let (buff, padding_zeros) = read_bytes(self.root_buff.clone())?;

            self.root_buff.advance(buff.len() + 32 + padding_zeros);

            Ok(buff)
        }
    }

    /// Returns remaining buff data.
    pub fn remaining(&self) -> Bytes {
        if let Some(tuple) = self.tuple_stacks.last() {
            tuple.remaining()
        } else {
            self.root_buff.clone()
        }
    }

    pub fn read_usize(&mut self) -> Result<usize, AbiDeError> {
        let buff = self.read_static()?;

        let usize_len = usize::BITS as usize / 8;

        Ok(usize::from_be_bytes(
            buff[(32 - usize_len)..].try_into().unwrap(),
        ))
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut AbiDeserializer {
    type Error = AbiDeError;

    fn is_human_readable(&self) -> bool {
        false
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!(
            "The ethereum contract abi is not a self-describing format. don't call deserialize_any"
        )
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        if buff[31] == 1 {
            visitor.visit_bool(true)
        } else {
            visitor.visit_bool(false)
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_byte_buf(self.read_bytes()?.to_vec())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bytes(&self.read_bytes()?)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("Contract abi don't support rust char")
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("Contract abi don't support rust enum")
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("Contract abi don't support rust f32")
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("Contract abi don't support rust f64")
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = i128::BITS as usize / 8;

        visitor.visit_i128(i128::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = i64::BITS as usize / 8;

        visitor.visit_i64(i64::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = i32::BITS as usize / 8;

        visitor.visit_i32(i32::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = i16::BITS as usize / 8;

        visitor.visit_i16(i16::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = i8::BITS as usize / 8;

        visitor.visit_i8(i8::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("Contract abi don't support rust map")
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match name {
            "bytes" => {
                let buff = self.read_bytes()?;

                visitor.visit_byte_buf(buff.to_vec())
            }
            "address" => {
                let buff = self.read_static()?;

                return visitor.visit_byte_buf(buff.to_vec());
            }
            _ => {
                let bytes_regex = Regex::new(r"^bytes(\d{1,2})$").unwrap();
                let int_regex = Regex::new(r"^(u)?int(\d{1,3})$").unwrap();

                if let Some(caps) = bytes_regex.captures(name) {
                    let len: usize = caps[1].parse().unwrap();
                    if len <= 32 {
                        let buff = self.read_static()?;

                        return visitor.visit_byte_buf(buff.to_vec());
                    }
                }

                if let Some(caps) = int_regex.captures(name) {
                    let len: usize = caps[2].parse().unwrap();
                    if len <= 256 {
                        let buff = self.read_static()?;

                        return visitor.visit_byte_buf(buff.to_vec());
                    }
                }

                visitor.visit_newtype_struct(self)
            }
        }
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!("Contract abi don't support rust Option")
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let len = self
            .start_read_tuple(true)?
            .ok_or(AbiDeError::NextIsStatic("deserialize seq".to_owned()))?;

        visitor.visit_seq(TupleAccess { de: self, len })
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_bytes()?;
        visitor.visit_str(&String::from_utf8(buff.to_vec()).map_err(de::Error::custom)?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_bytes()?;
        visitor.visit_string(String::from_utf8(buff.to_vec()).map_err(de::Error::custom)?)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.start_read_tuple(false)?;

        visitor.visit_seq(TupleAccess { de: self, len })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = u128::BITS as usize / 8;

        visitor.visit_u128(u128::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = u64::BITS as usize / 8;

        visitor.visit_u64(u64::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = u32::BITS as usize / 8;

        visitor.visit_u32(u32::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = u16::BITS as usize / 8;

        visitor.visit_u16(u16::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let buff = self.read_static()?;

        let len = u8::BITS as usize / 8;

        visitor.visit_u8(u8::from_be_bytes(buff[(32 - len)..].try_into().unwrap()))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct TupleAccess<'a> {
    de: &'a mut AbiDeserializer,
    len: usize,
}

impl<'de, 'a> de::SeqAccess<'de> for TupleAccess<'a> {
    type Error = AbiDeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.len > 0 {
            self.len -= 1;
            seed.deserialize(&mut *self.de).map(|c| Some(c))
        } else {
            self.de.end_read_tuple()?;

            Ok(None)
        }
    }
}

/// Deserialize rust value from contract abi format.
pub fn from_abi<'de, D: Deserialize<'de>, B: Into<Bytes>>(data: B) -> Result<D, AbiDeError> {
    let mut deserializer = AbiDeserializer::new(data.into());

    D::deserialize(&mut deserializer)
}
