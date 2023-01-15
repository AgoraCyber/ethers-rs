use ethers_rs::contract;

contract!(PersonalWallet, file = "tests/wallet.json");

#[allow(unused)]
#[async_std::test]
async fn test_weth() {
    let mut wallet =
        PersonalWallet::new("0xdD2FD4581271e230360230F9337D5c0430Bf44C0").expect("Create wallet");

    // let address = wallet.weth().await.expect("Get weth address");
}
