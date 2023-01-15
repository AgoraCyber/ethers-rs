use ethers_rs::{Address, Contract};

#[derive(Debug, Default, Contract)]
#[abi_file("tests/wallet.json")]
pub struct PersonalWallet(Option<Address>);
