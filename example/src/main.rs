use ethers_rs::{
    hardhat::{cmds::HardhatNetwork, utils::*},
    *,
};

hardhat!(Lock);

#[async_std::main]
async fn main() {
    _ = pretty_env_logger::try_init();

    let mut network = HardhatNetwork::new().expect("Create hardhat network instance");

    network.start().await.expect("Start hardhat network");

    let s0 = get_hardhat_network_account(0);

    let provider = get_hardhat_network_provider();

    let value = "1.1".parse::<Ether>().expect("Parse payment eth value");

    let client: Client = (provider.clone(), s0).into();

    // Deploy contract and send ether to deployed
    let lock = Lock::deploy_contract(client.clone(), 1usize, value.to_tx_options())
        .await
        .expect("Deploy lock contract");

    log::debug!("deploy lock success, {}", lock.address);
}
