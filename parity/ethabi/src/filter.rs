// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::{ops, str::FromStr};

use ethereum_types::H256;
use serde::{de::Visitor, Deserialize};
#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;
use crate::{Error, Hash, Token};

/// Raw topic filter.
#[derive(Debug, PartialEq, Default)]
pub struct RawTopicFilter {
    /// Topic.
    pub topic0: Topic<Token>,
    /// Topic.
    pub topic1: Topic<Token>,
    /// Topic.
    pub topic2: Topic<Token>,
}

/// Topic filter.
#[derive(Debug, PartialEq, Eq, Default, Clone, Hash)]
pub struct TopicFilter {
    /// Usually (for not-anonymous transactions) the first topic is event signature.
    pub topic0: Topic<Hash>,
    /// Second topic.
    pub topic1: Topic<Hash>,
    /// Third topic.
    pub topic2: Topic<Hash>,
    /// Fourth topic.
    pub topic3: Topic<Hash>,
}

#[cfg(feature = "serde")]
impl Serialize for TopicFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut data = vec![];

        if self.topic0.need_serialize() {
            data.push(&self.topic0)
        }

        if self.topic1.need_serialize() {
            data.push(&self.topic1)
        }

        if self.topic2.need_serialize() {
            data.push(&self.topic2)
        }

        if self.topic3.need_serialize() {
            data.push(&self.topic3)
        }

        data.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for TopicFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let topics = Vec::<Topic<Hash>>::deserialize(deserializer)?;

        if topics.len() > 4 || topics.is_empty() {
            return Err(Error::Other("Topics > 4 or Topics == 0".into()))
                .map_err(serde::de::Error::custom);
        } else {
            let none = Topic::<Hash>::Any;
            let topic0 = topics[0].clone();

            let topic1 = topics.get(1).unwrap_or(&none).clone();

            let topic2 = topics.get(2).unwrap_or(&none).clone();

            let topic3 = topics.get(3).unwrap_or(&none).clone();

            Ok(TopicFilter {
                topic0,
                topic1,
                topic2,
                topic3,
            })
        }
    }
}

/// Acceptable topic possibilities.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Topic<T> {
    /// Should skip serialize.
    None,
    /// Match any.
    Any,
    /// Match any of the hashes.
    OneOf(Vec<T>),
    /// Match only this hash.
    This(T),
}

impl<T> Topic<T> {
    /// Map
    pub fn map<F, O>(self, f: F) -> Topic<O>
    where
        F: Fn(T) -> O,
    {
        match self {
            Topic::None => Topic::None,
            Topic::Any => Topic::Any,
            Topic::OneOf(topics) => Topic::OneOf(topics.into_iter().map(f).collect()),
            Topic::This(topic) => Topic::This(f(topic)),
        }
    }

    /// Returns true if topic is empty (Topic::Any)
    pub fn is_any(&self) -> bool {
        match *self {
            Topic::Any => true,
            Topic::None | Topic::This(_) | Topic::OneOf(_) => false,
        }
    }
    /// Returns true if topic should be serialized
    pub fn need_serialize(&self) -> bool {
        match *self {
            Topic::None => false,
            _ => true,
        }
    }
}

impl<T> Default for Topic<T> {
    fn default() -> Self {
        Topic::None
    }
}

impl<T> From<Option<T>> for Topic<T> {
    fn from(o: Option<T>) -> Self {
        match o {
            Some(topic) => Topic::This(topic),
            None => Topic::Any,
        }
    }
}

impl<T> From<T> for Topic<T> {
    fn from(topic: T) -> Self {
        Topic::This(topic)
    }
}

impl<T> From<Vec<T>> for Topic<T> {
    fn from(topics: Vec<T>) -> Self {
        Topic::OneOf(topics)
    }
}

impl<T> From<Topic<T>> for Vec<T> {
    fn from(topic: Topic<T>) -> Self {
        match topic {
            Topic::None => vec![],
            Topic::Any => vec![],
            Topic::This(topic) => vec![topic],
            Topic::OneOf(topics) => topics,
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Topic<Hash> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Topic::Any | Topic::None => Option::<()>::None.serialize(serializer),
            Topic::OneOf(ref vec) => vec.serialize(serializer),
            Topic::This(ref hash) => hash.serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Topic<Hash> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TopicVisitor;

        impl<'de> Visitor<'de> for TopicVisitor {
            type Value = Topic<Hash>;
            fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
                write!(formatter, "Expect null/h256/[h256]")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let topic = H256::from_str(v).map_err(serde::de::Error::custom)?;

                Ok(Topic::This(topic))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut topics = vec![];

                while let Some(el) = seq.next_element::<&str>()? {
                    let topic = H256::from_str(el).map_err(serde::de::Error::custom)?;

                    topics.push(topic);
                }

                Ok(Topic::OneOf(topics))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Topic::Any)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Topic::Any)
            }
        }

        deserializer.deserialize_any(TopicVisitor {})
    }
}

impl<T> ops::Index<usize> for Topic<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match *self {
            Topic::Any | Topic::None => panic!("Topic unavailable"),
            Topic::This(ref topic) => {
                if index != 0 {
                    panic!("Topic unavailable");
                }
                topic
            }
            Topic::OneOf(ref topics) => topics.index(index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Topic;
    #[cfg(feature = "serde")]
    use super::TopicFilter;
    #[cfg(not(feature = "std"))]
    use crate::no_std_prelude::*;
    #[cfg(feature = "serde")]
    use crate::Hash;

    #[cfg(feature = "serde")]
    fn hash(s: &'static str) -> Hash {
        s.parse().unwrap()
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_topic_filter_serialization() {
        let expected = r#"["0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b",null,["0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b","0x0000000000000000000000000aff3454fce5edbc8cca8697c15331677e6ebccc"],null]"#;

        let topic = TopicFilter {
            topic0: Topic::This(hash(
                "000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b",
            )),
            topic1: Topic::Any,
            topic2: Topic::OneOf(vec![
                hash("000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b"),
                hash("0000000000000000000000000aff3454fce5edbc8cca8697c15331677e6ebccc"),
            ]),
            topic3: Topic::Any,
        };

        let topic_str = serde_json::to_string(&topic).unwrap();

        assert_eq!(expected, &topic_str);

        let expected_topic: TopicFilter = serde_json::from_str(&topic_str).unwrap();

        assert_eq!(expected_topic, topic);
    }

    #[test]
    fn test_topic_from() {
        assert_eq!(Topic::Any as Topic<u64>, None.into());
        assert_eq!(Topic::This(10u64), 10u64.into());
        assert_eq!(Topic::OneOf(vec![10u64, 20]), vec![10u64, 20].into());
    }

    #[test]
    fn test_topic_into_vec() {
        let expected: Vec<u64> = vec![];
        let is: Vec<u64> = (Topic::Any as Topic<u64>).into();
        assert_eq!(expected, is);
        let expected: Vec<u64> = vec![10];
        let is: Vec<u64> = Topic::This(10u64).into();
        assert_eq!(expected, is);
        let expected: Vec<u64> = vec![10, 20];
        let is: Vec<u64> = Topic::OneOf(vec![10u64, 20]).into();
        assert_eq!(expected, is);
    }

    #[test]
    fn test_topic_is_any() {
        assert!((Topic::Any as Topic<u8>).is_any());
        assert!(!Topic::OneOf(vec![10u64, 20]).is_any());
        assert!(!Topic::This(10u64).is_any());
    }

    #[test]
    fn test_topic_index() {
        assert_eq!(Topic::OneOf(vec![10u64, 20])[0], 10);
        assert_eq!(Topic::OneOf(vec![10u64, 20])[1], 20);
        assert_eq!(Topic::This(10u64)[0], 10);
    }

    #[test]
    #[should_panic(expected = "Topic unavailable")]
    fn test_topic_index_panic() {
        let _ = (Topic::Any as Topic<u8>)[0];
    }

    #[test]
    #[should_panic(expected = "Topic unavailable")]
    fn test_topic_index_panic2() {
        assert_eq!(Topic::This(10u64)[1], 10);
    }
}
