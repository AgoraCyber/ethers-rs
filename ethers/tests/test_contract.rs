use ethers_rs::{Address, Contract};

#[derive(Contract, Debug, Default)]
#[abi_file("tests/wallet.json")]
pub struct PersonalWallet(Option<Address>);
