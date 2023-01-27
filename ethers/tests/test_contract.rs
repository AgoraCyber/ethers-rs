use ethers_hardhat_rs::{
    cmds::HardhatNetwork,
    utils::{get_hardhat_network_account, get_hardhat_network_provider},
};

use ethers_rs::{contract, ethabi::RawLog, Client, Ether, ToTxOptions};

// It is not necessary to specify hardhat artifacts path
contract!(Lock, hardhat = "sol/artifacts/contracts/Lock.sol/Lock.json");

#[async_std::test]
async fn test_deploy() {
    _ = pretty_env_logger::try_init();

    let mut network = HardhatNetwork::new().expect("Create hardhat network instance");

    network.start().await.expect("Start hardhat network");

    let s0 = get_hardhat_network_account(0);

    let provider = get_hardhat_network_provider();

    let value = "1.1".parse::<Ether>().expect("Parse payment eth value");

    let mut client: Client = (provider.clone(), s0).into();

    let mut lock = Lock::deploy(client.clone(), value.clone().to_tx_options())
        .await
        .expect("Deploy lock contract");

    log::debug!("deploy lock success, {}", lock.address());

    let mut event_receiver = provider
        .register_filter_listener(
            Some(lock.address()),
            Some(lock::events::withdrawal::wildcard_filter()),
        )
        .expect("Create withdrawal event filter");

    let balance = client.balance().await.expect("Get after deploy balance");

    let mut tx = lock.withdraw().await.expect("Try withdraw");

    let receipt = tx.wait().await.expect("Wait withdraw tx mint");

    log::debug!(
        "receipt {}",
        serde_json::to_string_pretty(&receipt).expect("")
    );

    let balance_after = client.balance().await.expect("Get after deploy balance");

    log::debug!("{} {}", Ether(balance), Ether(balance_after));

    assert_eq!(
        balance + value.clone() - receipt.gas_used * receipt.effective_gas_price,
        balance_after
    );

    let logs = event_receiver
        .try_next()
        .await
        .expect("Try receive next logs");

    assert_eq!(logs.is_some(), true);

    for log in logs.as_ref().unwrap() {
        assert_eq!(log.topics[0], lock::events::withdrawal::event().signature());

        let withdrawal = lock::events::withdrawal::parse_log(RawLog {
            topics: log.topics.clone(),
            data: log.data.0.clone(),
        })
        .expect("Parse withdrawal log");

        assert_eq!(withdrawal.amount, value.0);
    }

    log::debug!("withdraw logs {:?}", logs);
}
