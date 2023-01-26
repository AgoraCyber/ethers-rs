use ethers_hardhat_rs::{
    cmds::HardhatNetwork,
    utils::{get_hardhat_network_account, get_hardhat_network_provider},
};

use ethers_rs::{contract, Ether, ToTxOptions};

// It is not necessary to specify hardhat artifacts path
contract!(Lock);

#[async_std::test]
async fn test_deploy() {
    _ = pretty_env_logger::try_init();

    let mut network = HardhatNetwork::new().expect("Create hardhat network instance");

    network.start().await.expect("Start hardhat network");

    let s0 = get_hardhat_network_account(0);

    let provider = get_hardhat_network_provider();

    let value = "1.1".parse::<Ether>().expect("Parse payment eth value");

    let lock = Lock::deploy((provider, s0), "0x10000000000", value.to_tx_options())
        .await
        .expect("Deploy lock contract");

    log::debug!("deploy lock success, {}", lock.address());
}
