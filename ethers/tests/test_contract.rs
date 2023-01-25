use ethers_providers_rs::providers::http;
use ethers_rs::contract;
use ethers_signer_rs::wallet::WalletSigner;
use ethers_wallet_rs::wallet::Wallet;

contract!(Lock, hardhat = "sol/artifacts/contracts/Lock.sol/Lock.json");

#[async_std::test]
async fn test_weth() {
    _ = pretty_env_logger::try_init();

    let provider = http::connect_to("http://localhost:8545");

    let signer = Wallet::new("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
        .expect("Create local wallet")
        .try_into_signer()
        .expect("Convert local wallet into signer");

    Lock::deploy((provider, signer), "0x10000000000")
        .await
        .expect("Deploy lock contract");
}
