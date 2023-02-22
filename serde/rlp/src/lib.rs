use regex::Regex;
use serde::{ser, Serialize};

use thiserror::Error;

/// Abi serializer error variant
#[derive(Debug, Error)]
pub enum RlpError {
    #[error("Unknown error,{0}")]
    Unknown(String),

    #[error("Serilaize multi-element without calling start_list")]
    List,

    #[error("Call finalize without calling end_list")]
    UnclosedList,

    #[error("call end_list before calling start_list")]
    UnopenList,

    #[error("Unsupport serialize type, {0}")]
    UnsupportType(String),
}

impl ser::Error for RlpError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Unknown(msg.to_string())
    }
}

#[derive(Debug, Default)]
struct RlpList {
    buff: Vec<u8>,
}

impl RlpList {
    fn finalize(mut self) -> Result<Vec<u8>, RlpError> {
        let len = self.buff.len();
        if len <= 0xc0 {
            let mut buff = vec![0xc0 + len as u8];

            buff.append(&mut self.buff);

            Ok(buff)
        } else {
            let mut encoder = RlpEncoder::default();

            rlp_encode_usize(&mut encoder, len)?;

            let mut len_buff = encoder.finalize()?;

            let mut buff = vec![0xf7 + len_buff.len() as u8];

            buff.append(&mut len_buff);
            buff.append(&mut self.buff);

            Ok(buff)
        }
    }

    pub fn rlp_append_string(&mut self, bytes: &[u8]) -> Result<(), RlpError> {
        self.buff.append(&mut bytes.to_owned());

        Ok(())
    }
}

fn rlp_encode_string(bytes: &[u8]) -> Result<Vec<u8>, RlpError> {
    let len = bytes.len();

    match len {
        0 => Ok(vec![0x80u8]),
        len @ 1..=55 => {
            if len == 1 && bytes[0] < 0x80 {
                Ok(bytes.to_owned())
            } else {
                let mut buff = vec![0x80u8 + len as u8];

                buff.append(&mut bytes.to_owned());

                Ok(buff)
            }
        }
        len => {
            let mut encoder = RlpEncoder::default();

            rlp_encode_usize(&mut encoder, len)?;

            let mut len_buff = encoder.finalize()?;

            let mut buff = vec![0xb7 + len_buff.len() as u8];

            buff.append(&mut len_buff);
            buff.append(&mut bytes.to_owned());

            Ok(buff)
        }
    }
}

fn rlp_encode_usize(encoder: &mut RlpEncoder, value: usize) -> Result<(), RlpError> {
    let offset: usize = value.leading_zeros() as usize / 8;

    encoder.append_string(&value.to_be_bytes()[offset..])?;

    Ok(())
}

/// Rlp(RECURSIVE-LENGTH PREFIX) format stream like encoder.
#[derive(Debug, Default)]
pub struct RlpEncoder {
    buff: Vec<u8>,
    list_stack: Vec<RlpList>,
}

impl RlpEncoder {
    /// Start a new encoding round for list item
    pub fn start_list(&mut self) -> Result<(), RlpError> {
        self.list_stack.push(Default::default());

        Ok(())
    }

    /// In [`rlp`](https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/) format,
    /// `string` means **"a certain number of bytes of binary data"**; no special encodings are used,
    /// and no knowledge about the content of the strings is implied.
    pub fn append_string(&mut self, bytes: &[u8]) -> Result<(), RlpError> {
        if !self.list_stack.is_empty() {
            self.list_stack
                .last_mut()
                .unwrap()
                .rlp_append_string(&rlp_encode_string(bytes)?)?;
        } else {
            if self.buff.len() != 0 {
                return Err(RlpError::List.into());
            } else {
                self.buff = rlp_encode_string(bytes)?;
            }
        }
        Ok(())
    }

    /// End the lastest encoding round for list item which is openned by fn [`start_list`](RlpEncoder::start_list)
    ///
    /// Call [`start_list`](RlpEncoder::start_list) first before calling this fn
    pub fn end_list(&mut self) -> Result<(), RlpError> {
        if let Some(list) = self.list_stack.pop() {
            let mut buff = list.finalize()?;

            // check if in a list
            if !self.list_stack.is_empty() {
                self.list_stack
                    .last_mut()
                    .unwrap()
                    .rlp_append_string(&buff)?;
            } else {
                self.buff.append(&mut buff);
            }

            return Ok(());
        } else {
            return Err(RlpError::UnopenList.into());
        }
    }

    /// Close encoder and return result bytes.
    pub fn finalize(self) -> Result<Vec<u8>, RlpError> {
        if !self.list_stack.is_empty() {
            return Err(RlpError::UnclosedList.into());
        }

        Ok(self.buff)
    }
}

fn signed_to_buff(bytes: &[u8]) -> &[u8] {
    let lead_ones = bytes.iter().take_while(|c| **c == 0xff).count();
    let lead_zeros = bytes.iter().take_while(|c| **c == 0x00).count();

    if lead_ones > 0 {
        &bytes[(lead_ones - 1)..]
    } else {
        &bytes[lead_zeros..]
    }
}

impl<'a> ser::Serializer for &'a mut RlpEncoder {
    type Ok = ();
    type Error = RlpError;

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
        Err(RlpError::UnsupportType("bool".to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.append_string(v)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!("Contract abi don't support rust char")
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(RlpError::UnsupportType("f64".to_owned()))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_be_bytes();

        self.append_string(signed_to_buff(&bytes))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_be_bytes();

        self.append_string(signed_to_buff(&bytes))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_be_bytes();

        self.append_string(signed_to_buff(&bytes))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_be_bytes();

        self.append_string(signed_to_buff(&bytes))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_be_bytes();

        self.append_string(signed_to_buff(&bytes))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(RlpError::UnsupportType("map".to_owned()))
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

                self.append_string(bytes)
            }
            "address" => {
                let bytes = unsafe { (value as *const T).cast::<[u8; 32]>().as_ref().unwrap() };

                return self.append_string(&bytes[16..]);
            }
            _ => {
                let bytes_regex = Regex::new(r"^bytes(\d{1,2})$").unwrap();
                let int_regex = Regex::new(r"^(u)?int(\d{1,3})$").unwrap();

                if let Some(caps) = bytes_regex.captures(name) {
                    let len: usize = caps[1].parse().unwrap();
                    if len <= 32 {
                        let bytes =
                            unsafe { (value as *const T).cast::<[u8; 32]>().as_ref().unwrap() };

                        return self.append_string(&bytes[..len]);
                    }
                }

                if let Some(caps) = int_regex.captures(name) {
                    let len: usize = caps[2].parse().unwrap();
                    if len <= 256 {
                        let bytes =
                            unsafe { (value as *const T).cast::<[u8; 32]>().as_ref().unwrap() };

                        if caps.get(1).is_some() {
                            let lead_zeros = bytes.iter().take_while(|c| **c == 0).count();
                            let buff = &bytes[lead_zeros..];

                            return self.append_string(buff);
                        } else {
                            return self.append_string(signed_to_buff(bytes));
                        }
                    }
                }

                value.serialize(self)
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
        self.start_list()?;

        Ok(self)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        unimplemented!("Contract abi don't support rust Option")
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.append_string(v.as_bytes())
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
        self.start_list()?;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.start_list()?;
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

        let leading_zeros = v.leading_zeros() as usize / 8;

        self.append_string(&bytes[leading_zeros..])
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_be_bytes();

        let leading_zeros = v.leading_zeros() as usize / 8;

        self.append_string(&bytes[leading_zeros..])
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_be_bytes();

        let leading_zeros = v.leading_zeros() as usize / 8;

        self.append_string(&bytes[leading_zeros..])
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let bytes = v.to_be_bytes();

        let leading_zeros = v.leading_zeros() as usize / 8;

        self.append_string(&bytes[leading_zeros..])
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.append_string(&[v])
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

impl<'a> ser::SerializeMap for &'a mut RlpEncoder {
    type Error = RlpError;

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

impl<'a> ser::SerializeSeq for &'a mut RlpEncoder {
    type Error = RlpError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_list()?;

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

impl<'a> ser::SerializeStruct for &'a mut RlpEncoder {
    type Error = RlpError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_list()?;

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

impl<'a> ser::SerializeStructVariant for &'a mut RlpEncoder {
    type Error = RlpError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_list()?;

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

impl<'a> ser::SerializeTuple for &'a mut RlpEncoder {
    type Error = RlpError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_list()?;

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

impl<'a> ser::SerializeTupleVariant for &'a mut RlpEncoder {
    type Error = RlpError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_list()?;

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

impl<'a> ser::SerializeTupleStruct for &'a mut RlpEncoder {
    type Error = RlpError;

    type Ok = ();
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_list()?;

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

/// Serialize rust value to rlp format.
pub fn rlp_encode<S: Serialize + ?Sized>(value: &S) -> anyhow::Result<Vec<u8>> {
    let mut serializer = RlpEncoder::default();

    value.serialize(&mut serializer).expect("");

    Ok(serializer.finalize()?)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_rlp_encode() {
        assert_eq!(rlp_encode("dog").unwrap(), [0x83u8, b'd', b'o', b'g']);

        assert_eq!(
            rlp_encode(&vec!["cat".to_string(), "dog".to_string()]).unwrap(),
            [0xc8, 0x83u8, b'c', b'a', b't', 0x83, b'd', b'o', b'g']
        );

        assert_eq!(rlp_encode(&Vec::<String>::new()).unwrap(), [0xc0]);

        assert_eq!(rlp_encode(&0usize).unwrap(), [0x80]);

        assert_eq!(rlp_encode(&0usize).unwrap(), [0x80]);

        assert_eq!(rlp_encode("\x00").unwrap(), [0x00]);

        assert_eq!(rlp_encode("\x04\x00").unwrap(), [0x82, 0x04, 0x00]);

        fn test_lists() -> Vec<u8> {
            let mut encoder = RlpEncoder::default();

            encoder.start_list().unwrap();

            {
                encoder.start_list().unwrap();
                encoder.end_list().unwrap();
            }

            {
                encoder.start_list().unwrap();

                {
                    encoder.start_list().unwrap();
                    encoder.end_list().unwrap();
                }

                encoder.end_list().unwrap();
            }

            {
                encoder.start_list().unwrap();

                {
                    encoder.start_list().unwrap();
                    encoder.end_list().unwrap();
                }

                {
                    encoder.start_list().unwrap();

                    {
                        encoder.start_list().unwrap();
                        encoder.end_list().unwrap();
                    }

                    encoder.end_list().unwrap();
                }

                encoder.end_list().unwrap();
            }

            encoder.end_list().unwrap();

            encoder.finalize().unwrap()
        }

        assert_eq!(
            test_lists(),
            [0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0]
        );

        let mut expected = [0xb8, 0x38].to_vec();

        expected.append(&mut b"Lorem ipsum dolor sit amet, consectetur adipisicing elit".to_vec());

        assert_eq!(
            rlp_encode("Lorem ipsum dolor sit amet, consectetur adipisicing elit").unwrap(),
            expected
        );
    }
}
