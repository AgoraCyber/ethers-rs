use std::str::FromStr;

use ethabi::Address;
use ethers_rs::contract;

// #[derive(Debug, Default, Contract)]
// #[abi_file("tests/wallet.json")]
// pub struct PersonalWallet(Option<Address>);

contract!(PersonalWallet, file = "tests/wallet.json");

#[allow(unused)]
#[async_std::test]
async fn test_weth() {
    let mut wallet = PersonalWallet::new(Address::from_str("0x0").unwrap());

    let address = wallet.weth().await.expect("Get weth address");
}
