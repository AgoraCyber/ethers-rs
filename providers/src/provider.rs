use std::sync::{Arc, Mutex};

mod event;
pub use event::*;
mod rpc;
pub use rpc::*;

/// Ether network api provider
///
#[derive(Clone)]
#[allow(dead_code)]
pub struct Provider {
    id: String,
    rpc_client: jsonrpc_rs::Client,
    pub(crate) oneshot: OneshotCompleteQ,
    pub(crate) channel: ChannelCompleteQ,
    pub(crate) events: Arc<Mutex<Vec<EventType>>>,
}

impl Provider {
    pub fn new(id: String, rpc_client: jsonrpc_rs::Client) -> Self {
        let this = Self {
            id,
            oneshot: OneshotCompleteQ::new(),
            channel: ChannelCompleteQ::new(),
            events: Default::default(),
            rpc_client,
        };

        this.start_event_poll();

        this
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get provider inner jsonrpc [`Client`](jsonrpc_rs::Client) ref.
    pub fn client(&mut self) -> &mut jsonrpc_rs::Client {
        &mut self.rpc_client
    }
}

#[cfg(test)]
mod tests {

    use crate::types::*;
    use jsonrpc_rs::RPCResult;
    use serde_json::json;

    use crate::impls::http;

    #[async_std::test]
    async fn test_block_number() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let block_number = provider.eth_block_number().await?;

        log::debug!("block number {}", block_number);

        Ok(())
    }

    #[async_std::test]
    async fn test_chain_id() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let block_number = provider.eth_chain_id().await?;

        log::debug!("chain_id {}", block_number);

        Ok(())
    }

    #[async_std::test]
    async fn test_eth_getblockbynumber() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let block = provider.eth_get_block_by_number("latest", true).await?;

        log::debug!(
            "block
             {}",
            serde_json::to_string(&block).expect("Serialize block to json")
        );

        let block = provider.eth_get_block_by_number("0x1", true).await?;

        log::debug!(
            "block
             {}",
            serde_json::to_string(&block).expect("Serialize block to json")
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_eth_getblocktransactioncountbyhash() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let block = provider
            .eth_get_block_by_number("pending", true)
            .await?
            .unwrap();

        if let Some(hash) = block.hash {
            let tx_count = provider
                .eth_get_block_transaction_count_by_hash(hash.clone())
                .await?;

            assert_eq!(Some(block.transactions.len()), tx_count.into());

            let uncles = provider
                .eth_get_uncle_count_by_block_hash(hash.clone())
                .await?;

            assert_eq!(Some(block.uncles.len()), uncles.into());

            if let Some(number) = block.number {
                let uncles = provider.eth_get_uncle_count_by_block_number(number).await?;

                assert_eq!(Some(block.uncles.len()), uncles.into());
            }
        }

        Ok(())
    }

    #[async_std::test]
    async fn test_eth_syncing() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let status = provider.eth_syncing().await?;

        assert_eq!(status, SyncingStatus::False);

        Ok(())
    }

    #[async_std::test]
    async fn test_eth_coinbase() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let address = provider.eth_coinbase().await.expect("eth_coinbase");

        log::debug!("{}", address);
    }

    #[async_std::test]
    async fn test_eth_accounts() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let accounts = provider.eth_accounts().await.expect("eth_coinbase");

        log::debug!(
            "{:?}",
            accounts.iter().map(|a| a.to_string()).collect::<Vec<_>>()
        );
    }

    #[async_std::test]
    async fn test_eth_gas_price() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let price = provider.eth_gas_price().await.expect("eth_gas_price");

        log::debug!("{}", price);
    }

    #[async_std::test]
    async fn test_eth_free_history() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let history = provider
            .eth_fee_history("0x10", "latest", &[100f64])
            .await
            .expect("eth_fee_history");

        log::debug!(
            "{}",
            serde_json::to_string_pretty(&history).expect("Serialize history to json")
        );
    }

    #[async_std::test]
    async fn test_eth_new_filter() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        provider
            .eth_new_filter(json!({
                "fromBlock": "0x1",
                "toBlock": "0x02",
            }))
            .await
            .expect("eth_new_filter");
    }

    #[async_std::test]
    async fn test_eth_new_any_filter() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let filter = provider
            .eth_new_block_filter()
            .await
            .expect("eth_newBlockFilter");

        log::debug!("create new blockFilter {}", filter);

        let filter = provider
            .eth_new_pending_transaction_filter()
            .await
            .expect("eth_newPendingTransactionFilter");

        log::debug!("create new pending transaction filter {}", filter);

        let result = provider
            .eth_get_filter_changes(filter.clone())
            .await
            .expect("eth_getFilterChanges");

        log::debug!("eth_getFilterChanges {:?}", result);

        assert_eq!(
            provider
                .eth_uninstall_filter(filter)
                .await
                .expect("eth_uninstallFilter"),
            true
        );
    }

    #[async_std::test]
    async fn test_eth_get_logs() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        provider
            .eth_new_filter(json!({
                "fromBlock": "0x1",
                "toBlock": "0x02",
            }))
            .await
            .expect("eth_new_filter");

        let result = provider
            .eth_get_logs(json!({
                "fromBlock": "0x1",
                "toBlock": "0x02",
            }))
            .await
            .expect("eth_getLogs");

        log::debug!("eth_getFilterChanges {:?}", result);
    }

    // #[async_std::test]
    // async fn test_eth_sign() {
    //     _ = pretty_env_logger::try_init();

    //     let mut provider = http::connect_to("http://localhost:8545");

    //     let sig = provider
    //         .eth_sign(
    //             "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
    //             "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
    //         )
    //         .await
    //         .expect("eth_new_filter");

    //     log::debug!("eth_getFilterChanges {:?}", sig);
    // }

    // #[async_std::test]
    // async fn test_eth_sign_transaction() {
    //     _ = pretty_env_logger::try_init();

    //     let mut provider = http::connect_to("http://localhost:8545");

    //     let balance = provider
    //         .eth_get_balance("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")
    //         .await
    //         .expect("eth_getBlance");

    //     log::debug!("balance {}", balance);

    //     let nonce = provider
    //         .eth_get_transaction_count("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")
    //         .await
    //         .expect("eth_getTransactionCount");

    //     log::debug!("nonce {}", nonce);

    //     provider
    //         .eth_sign_transaction(json!({
    //             "nonce": nonce,
    //             "from": "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
    //             "to": "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
    //             "value":"0x11",
    //             "gas": "0x11",
    //             "input":"0x0"
    //         }))
    //         .await
    //         .expect_err("eth_signTransaction not support");
    // }

    #[async_std::test]
    async fn test_eth_send_raw_transaction() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        provider
            .eth_send_raw_transaction("0x000")
            .await
            .expect_err("eth_sendRawTransaction error");
    }

    #[async_std::test]
    async fn test_eth_get_transaction_by_hash() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        provider
            .eth_get_transaction_by_hash(
                "0x0bb3c2388383f714a8070dc6078a5edbe78f23c96646d4148d63cf964197ccc5",
            )
            .await
            .expect("eth_getTransactionByHash");
    }

    #[async_std::test]
    async fn test_eth_get_transaction_receipt() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        provider
            .eth_get_transaction_receipt(
                "0x0bb3c2388383f714a8070dc6078a5edbe78f23c96646d4148d63cf964197ccc5",
            )
            .await
            .expect("eth_getTransactionByHash");
    }
}
