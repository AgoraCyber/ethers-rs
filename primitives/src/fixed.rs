use serde::{Deserialize, Serialize};

pub struct Fixed<const M: usize, const N: usize>;

pub struct Unfixed<const M: usize, const N: usize>;

impl<const M: usize, const N: usize> Serialize for Unfixed<M, N> {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unimplemented!("ethers_rs don't support fixed<M,N>")
    }
}

impl<'de, const M: usize, const N: usize> Deserialize<'de> for Unfixed<M, N> {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!("ethers_rs don't support unfixed<M,N>")
    }
}

impl<const M: usize, const N: usize> Serialize for Fixed<M, N> {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        unimplemented!("ethers_rs don't support unfixed<M,N>")
    }
}

impl<'de, const M: usize, const N: usize> Deserialize<'de> for Fixed<M, N> {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        unimplemented!("ethers_rs don't support fixed<M,N>")
    }
}
