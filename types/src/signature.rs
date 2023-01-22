use ethabi::ethereum_types::{Signature, U256};

pub trait SignatureVRS {
    fn v(&self) -> u8;

    fn r(&self) -> U256;

    fn s(&self) -> U256;
}

impl SignatureVRS for Signature {
    /// Extract signature v
    fn v(&self) -> u8 {
        self.0[0]
    }

    /// Extract signature r
    fn r(&self) -> U256 {
        self.0[1..33].into()
    }

    /// Extract signature s
    fn s(&self) -> U256 {
        self.0[33..].into()
    }
}
