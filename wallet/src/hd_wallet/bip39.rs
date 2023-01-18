use std::{
    collections::HashMap,
    io::Write,
    ops::{BitAnd, BitOrAssign, DivAssign, MulAssign},
    str::FromStr,
};

use ethers_utils_rs::hex::bytes_to_hex;
use num::{BigUint, FromPrimitive};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// The minimum number of words in a mnemonic.
#[allow(unused)]
const MIN_NB_WORDS: usize = 12;

/// The maximum number of words in a mnemonic.
const MAX_NB_WORDS: usize = 24;

#[derive(Debug, Error)]
pub enum Bip39Error {
    #[error("Invalid entropy length {0}, expect entropy len [16,64] and a multiple of 4")]
    InvalidEntropyLength(usize),

    #[error("Invalid num of mnemonic words {0}, should be 12,15,18,21 or 24")]
    MnemonicNumOfWords(usize),

    #[error("Invalid mnemonic word {0}, not found in dictionary")]
    MnemonicWord(String),

    #[error("mnemonic checksum mismatch")]
    Checksum,
}

#[derive(Debug, Clone)]
pub struct Dictionary {
    word_list: Vec<String>,
    indexer: HashMap<String, usize>,
}

impl Dictionary {
    pub fn new(word_list: Vec<String>) -> Self {
        let mut indexer = HashMap::<String, usize>::new();

        for (index, word) in word_list.iter().enumerate() {
            indexer.insert(word.to_owned(), index);
        }

        Self { word_list, indexer }
    }
}

impl FromStr for Dictionary {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(serde_json::from_str(s)?))
    }
}

/// The Generator of entropy for random mnemonic
pub trait EntropyGenerator<const LEN: usize> {
    fn gen() -> Result<[u8; LEN], Bip39Error>;
}

/// Generator using [`rand::rngs::OsRng`] to generate `entropy`
pub struct OsrngEntropyGenerator;

impl<const LEN: usize> EntropyGenerator<LEN> for OsrngEntropyGenerator {
    fn gen() -> Result<[u8; LEN], Bip39Error> {
        if LEN % 4 != 0 {
            return Err(Bip39Error::InvalidEntropyLength(LEN));
        }

        let mut buff = [0; LEN];

        for i in 0..LEN / 4 {
            let random_u32 = OsRng.next_u32();

            let bytes = random_u32.to_be_bytes();

            log::debug!("generate random u32 {} {:x?}", random_u32, bytes);

            buff[i * 4] = bytes[0];
            buff[i * 4 + 1] = bytes[1];
            buff[i * 4 + 2] = bytes[2];
            buff[i * 4 + 3] = bytes[3];
        }

        log::debug!("{:x?}", buff);

        Ok(buff)
    }
}

#[derive(Debug, Clone)]
pub struct Bip39Generator {
    dic: Dictionary,
}

impl Bip39Generator {
    pub fn new(dic: Dictionary) -> Self {
        Self { dic }
    }

    pub fn gen_mnemonic<const LEN: usize>(&self) -> Result<String, Bip39Error> {
        self.gen_mnemonic_with::<OsrngEntropyGenerator, LEN>()
    }

    pub fn gen_mnemonic_with<EG, const LEN: usize>(&self) -> Result<String, Bip39Error>
    where
        EG: EntropyGenerator<LEN>,
    {
        if LEN < 16 || LEN > 64 || LEN % 4 != 0 {
            return Err(Bip39Error::InvalidEntropyLength(LEN));
        }

        let entropy = EG::gen()?;

        log::debug!("entropy {}", bytes_to_hex(&entropy));

        const MAX_ENTROPY_BITS: usize = 256;
        const MIN_ENTROPY_BITS: usize = 128;
        const MAX_CHECKSUM_BITS: usize = 8;

        let nb_bytes = entropy.len();
        let nb_bits = nb_bytes * 8;

        if nb_bits % 32 != 0 {
            return Err(Bip39Error::InvalidEntropyLength(LEN));
        }
        if nb_bits < MIN_ENTROPY_BITS || nb_bits > MAX_ENTROPY_BITS {
            return Err(Bip39Error::InvalidEntropyLength(LEN));
        }

        let mut hasher = Sha256::new();

        hasher.write(&entropy).expect("Sha256 entropy");

        let check = hasher.finalize();

        let mut bits = [false; MAX_ENTROPY_BITS + MAX_CHECKSUM_BITS];
        for i in 0..nb_bytes {
            for j in 0..8 {
                bits[i * 8 + j] = (entropy[i] & (1 << (7 - j))) > 0;
            }
        }
        for i in 0..nb_bytes / 4 {
            bits[8 * nb_bytes + i] = (check[i / 8] & (1 << (7 - (i % 8)))) > 0;
        }

        let mut words = vec![];

        let nb_words = nb_bytes * 3 / 4;
        for i in 0..nb_words {
            let mut idx = 0;
            for j in 0..11 {
                if bits[i * 11 + j] {
                    idx += 1 << (10 - j);
                }
            }

            words.push(self.dic.word_list[idx].clone());
        }

        Ok(words.join(" "))
    }

    pub fn mnemonic_check<S>(&self, mnemonic: S) -> Result<(), Bip39Error>
    where
        S: AsRef<str>,
    {
        let mnemonic = mnemonic.as_ref();

        let words = mnemonic.split(" ").collect::<Vec<_>>();

        let num_of_words = words.len();

        if num_of_words % 3 != 0 || num_of_words < 12 || num_of_words > 24 {
            return Err(Bip39Error::MnemonicNumOfWords(num_of_words));
        }

        let mut bits = [false; MAX_NB_WORDS * 11];

        for (i, word) in words.iter().enumerate() {
            if let Some(index) = self.dic.indexer.get(*word) {
                for j in 0..11 {
                    bits[i * 11 + j] = index >> (10 - j) & 1 == 1;
                }
            } else {
                return Err(Bip39Error::MnemonicWord(word.to_string()));
            }
        }

        let mut entropy = [0u8; MAX_NB_WORDS / 3 * 4];
        let nb_bytes_entropy = num_of_words / 3 * 4;
        for i in 0..nb_bytes_entropy {
            for j in 0..8 {
                if bits[i * 8 + j] {
                    entropy[i] += 1 << (7 - j);
                }
            }
        }

        let mut hasher = Sha256::new();

        hasher.write(&entropy[0..nb_bytes_entropy]).expect("");

        let check = hasher.finalize();

        for i in 0..nb_bytes_entropy / 4 {
            if bits[8 * nb_bytes_entropy + i] != ((check[i / 8] & (1 << (7 - (i % 8)))) > 0) {
                return Err(Bip39Error::Checksum);
            }
        }

        Ok(())
    }
}

pub mod languages {
    use super::Dictionary;

    pub fn en_us() -> Dictionary {
        include_str!("bip39/en_us.json")
            .parse()
            .expect("Load dictionary")
    }
}

#[cfg(test)]
mod tests {
    use super::{languages::en_us, Bip39Generator};

    #[test]
    fn test_checksum() {
        let _ = pretty_env_logger::try_init();

        let gen = Bip39Generator::new(en_us());

        let mnemonic = gen.gen_mnemonic::<16>().expect("Generate mnemonic");

        log::debug!("mnemonic: \r\n {}", mnemonic);

        gen.mnemonic_check(mnemonic).expect("Mnemonic check");
    }
}
