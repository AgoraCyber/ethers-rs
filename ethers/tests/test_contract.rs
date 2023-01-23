use ethers_providers_rs::providers::http;
use ethers_rs::{bytes::bytes_to_string, contract, Eip55};
use ethers_signer_rs::wallet::WalletSigner;
use ethers_wallet_rs::wallet::Wallet;

contract!(PersonalWallet, file = "tests/wallet.json");

#[allow(unused)]
#[async_std::test]
async fn test_weth() {
    _ = pretty_env_logger::try_init();

    let provider = http::connect_to("http://localhost:8545");

    let mut wallet = PersonalWallet::new("0x5fbdb2315678afecb367f032d93f642f64180aa3", provider)
        .expect("Create wallet")
        .with_signer(
            Wallet::new("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .expect("Create local wallet")
                .try_into_signer()
                .expect("Convert local wallet into signer"),
        );

    let address = wallet.weth().await.expect("Get weth address");

    assert_eq!(
        address.to_checksum_string(),
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
    );

    wallet
        .transfer_ownership("0x70997970C51812dc3A010C7d01b50e0d17dc79C8")
        .await
        .expect("Transfer ownership");
}

#[test]
fn test_selector() {
    _ = pretty_env_logger::try_init();

    use ethers_rs::ethabi;

    assert_eq!(
        "0xe0af3616",
        bytes_to_string(ethabi::short_signature("_WETH", &[]))
    );
}
