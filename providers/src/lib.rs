use std::fmt::{Debug, Display};

use jsonrpc_rs::RPCResult;

use ethers_types_rs::*;

use ethers_types_rs::Address;

pub mod error;
pub mod providers;

pub use error::ProviderError;

pub type BlockHash = H256;

/// Ether network api provider
///
#[derive(Clone)]
#[allow(dead_code)]
pub struct Provider {
    rpc_client: jsonrpc_rs::Client,
}

impl Provider {
    pub fn new(rpc_client: jsonrpc_rs::Client) -> Self {
        Self { rpc_client }
    }

    /// Get provider inner jsonrpc [`Client`](jsonrpc_rs::Client) ref.
    pub fn client(&mut self) -> &mut jsonrpc_rs::Client {
        &mut self.rpc_client
    }

    /// Returns the number of most recent block.
    pub async fn eth_block_number(&mut self) -> RPCResult<U256> {
        self.rpc_client
            .call("eth_blockNumber", Vec::<String>::new())
            .await
    }

    /// Returns the chain ID of the current network
    pub async fn eth_chain_id(&mut self) -> RPCResult<U64> {
        self.rpc_client
            .call("eth_chainId", Vec::<String>::new())
            .await
    }

    /// Returns information about a block by hash.
    pub async fn eth_get_block_by_hash<B>(
        &mut self,
        block_hash: B,
        hydrated: bool,
    ) -> RPCResult<Option<Block>>
    where
        B: TryInto<BlockHash>,
        B::Error: Debug + Display,
    {
        let block_hash = block_hash.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getBlockByHash", (block_hash, hydrated))
            .await
    }

    /// Returns information about a block by number
    pub async fn eth_get_block_by_number<BT>(
        &mut self,
        block_number_or_tag: BT,
        hydrated: bool,
    ) -> RPCResult<Option<Block>>
    where
        BT: TryInto<BlockNumberOrTag>,
        BT::Error: Debug + Display,
    {
        let block_number_or_tag = block_number_or_tag
            .try_into()
            .map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getBlockByNumber", (block_number_or_tag, hydrated))
            .await
    }

    /// Returns the number of transactions in a block from a block matching the given block hash
    pub async fn eth_get_block_transaction_count_by_hash<H>(&mut self, hash: H) -> RPCResult<U64>
    where
        H: TryInto<BlockHash>,
        H::Error: Debug + Display,
    {
        let hash = hash.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getBlockTransactionCountByNumber", vec![hash])
            .await
    }

    /// Returns the number of uncles in a block from a block matching the given block hash
    pub async fn eth_get_uncle_count_by_block_hash<H>(&mut self, hash: H) -> RPCResult<U64>
    where
        H: TryInto<BlockHash>,
        H::Error: Debug + Display,
    {
        let hash = hash.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getUncleCountByBlockHash", vec![hash])
            .await
    }

    /// Returns the number of uncles in a block from a block matching the given block hash
    pub async fn eth_get_uncle_count_by_block_number<BT>(
        &mut self,
        block_number_or_tag: BT,
    ) -> RPCResult<U64>
    where
        BT: TryInto<BlockNumberOrTag>,
        BT::Error: Debug + Display,
    {
        let block_number_or_tag = block_number_or_tag
            .try_into()
            .map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getUncleCountByBlockNumber", vec![block_number_or_tag])
            .await
    }

    /// Returns an object with data about the sync status or false
    pub async fn eth_syncing(&mut self) -> RPCResult<SyncingStatus> {
        self.rpc_client
            .call("eth_syncing", Vec::<String>::new())
            .await
    }

    /// Returns the client coinbase address.
    pub async fn eth_coinbase(&mut self) -> RPCResult<Address> {
        self.rpc_client
            .call("eth_coinbase", Vec::<String>::new())
            .await
    }

    /// Returns a list of addresses owned by client.
    pub async fn eth_accounts(&mut self) -> RPCResult<Vec<Address>> {
        self.rpc_client
            .call("eth_accounts", Vec::<String>::new())
            .await
    }

    /// Executes a new message call immediately without creating a transaction on the block chain.
    pub async fn eth_call<TX, BT>(
        &mut self,
        transaction: TX,
        block_number_or_tag: Option<BT>,
    ) -> RPCResult<Bytecode>
    where
        TX: TryInto<TypedTransactionRequest>,
        TX::Error: Debug + Display,
        BT: TryInto<BlockNumberOrTag>,
        BT::Error: Debug + Display,
    {
        let transaction = transaction.try_into().map_err(jsonrpc_rs::map_error)?;

        if let Some(block_number_or_tag) = block_number_or_tag {
            let block_number_or_tag = block_number_or_tag
                .try_into()
                .map_err(jsonrpc_rs::map_error)?;

            self.rpc_client
                .call("eth_call", (transaction, block_number_or_tag))
                .await
        } else {
            self.rpc_client.call("eth_call", vec![transaction]).await
        }
    }

    /// Generates and returns an estimate of how much gas is necessary to allow the transaction to complete.
    pub async fn eth_estimate_gas<TX, BT>(
        &mut self,
        transaction: TX,
        block_number_or_tag: Option<BT>,
    ) -> RPCResult<U256>
    where
        TX: TryInto<TypedTransactionRequest>,
        TX::Error: Debug + Display,
        BT: TryInto<BlockNumberOrTag>,
        BT::Error: Debug + Display,
    {
        let transaction = transaction.try_into().map_err(jsonrpc_rs::map_error)?;

        if let Some(block_number_or_tag) = block_number_or_tag {
            let block_number_or_tag = block_number_or_tag
                .try_into()
                .map_err(jsonrpc_rs::map_error)?;

            self.rpc_client
                .call("eth_estimateGas", (transaction, block_number_or_tag))
                .await
        } else {
            self.rpc_client
                .call("eth_estimateGas", vec![transaction])
                .await
        }
    }

    /// Generates an access list for a transaction
    pub async fn eth_create_accesslist<TX, BT>(
        &mut self,
        transaction: TX,
        block_number_or_tag: Option<BT>,
    ) -> RPCResult<U256>
    where
        TX: TryInto<Transaction>,
        TX::Error: Debug + Display,
        BT: TryInto<BlockNumberOrTag>,
        BT::Error: Debug + Display,
    {
        let transaction = transaction.try_into().map_err(jsonrpc_rs::map_error)?;

        if let Some(block_number_or_tag) = block_number_or_tag {
            let block_number_or_tag = block_number_or_tag
                .try_into()
                .map_err(jsonrpc_rs::map_error)?;

            self.rpc_client
                .call("eth_createAccessList", (transaction, block_number_or_tag))
                .await
        } else {
            self.rpc_client
                .call("eth_createAccessList", vec![transaction])
                .await
        }
    }

    /// Returns the current price gas in wei.
    pub async fn eth_gas_price(&mut self) -> RPCResult<U256> {
        self.rpc_client
            .call("eth_gasPrice", Vec::<String>::new())
            .await
    }

    /// Returns the current maxPriorityFeePerGas per gas in wei.
    pub async fn eth_max_priority_fee_per_gas(&mut self) -> RPCResult<U256> {
        self.rpc_client
            .call("eth_maxPriorityFeePerGas", Vec::<String>::new())
            .await
    }

    /// Returns transaction base fee per gas and effective priority fee per gas for the requested/supported block range.
    pub async fn eth_fee_history<N, BT, RP>(
        &mut self,
        block_count: N,
        newest_block: BT,
        reward_percentiles: RP,
    ) -> RPCResult<FeeHistory>
    where
        N: TryInto<U256>,
        N::Error: Debug + Display,
        BT: TryInto<BlockNumberOrTag>,
        BT::Error: Debug + Display,
        RP: AsRef<[f64]>,
    {
        let block_count = block_count.try_into().map_err(jsonrpc_rs::map_error)?;

        let newest_block = newest_block.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call(
                "eth_feeHistory",
                (block_count, newest_block, reward_percentiles.as_ref()),
            )
            .await
    }

    /// Returns transaction base fee per gas and effective priority fee per gas for the requested/supported block range.
    pub async fn eth_new_filter<F>(&mut self, filter: F) -> RPCResult<U256>
    where
        F: TryInto<Filter>,
        F::Error: Debug + Display,
    {
        let filter = filter.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_newFilter", vec![filter]).await
    }

    /// Creates new filter in the node,to notify when a new block arrives.
    pub async fn eth_new_block_filter(&mut self) -> RPCResult<U256> {
        self.rpc_client
            .call("eth_newBlockFilter", Vec::<String>::new())
            .await
    }

    /// Creates new filter in the node,to notify when new pending transactions arrive.
    pub async fn eth_new_pending_transaction_filter(&mut self) -> RPCResult<U256> {
        self.rpc_client
            .call("eth_newPendingTransactionFilter", Vec::<String>::new())
            .await
    }

    /// Uninstalls a filter with given id
    pub async fn eth_uninstall_filter<N>(&mut self, id: N) -> RPCResult<bool>
    where
        N: TryInto<U256>,
        N::Error: Debug + Display,
    {
        let id = id.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_uninstallFilter", vec![id]).await
    }

    /// Polling method for a filter, which returns an arrya of logs which occurred since last poll

    pub async fn eth_get_filter_changes<N>(&mut self, id: N) -> RPCResult<Option<PollLogs>>
    where
        N: TryInto<U256>,
        N::Error: Debug + Display,
    {
        let id = id.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_getFilterChanges", vec![id]).await
    }

    /// Returns any arrays of all logs matching filter with given id
    pub async fn eth_get_filter_logs<N>(&mut self, id: N) -> RPCResult<Option<PollLogs>>
    where
        N: TryInto<U256>,
        N::Error: Debug + Display,
    {
        let id = id.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_getFilterLogs", vec![id]).await
    }

    /// Returns an array of all logs matching filter with filter description
    pub async fn eth_get_logs<F>(&mut self, filter: F) -> RPCResult<Option<PollLogs>>
    where
        F: TryInto<Filter>,
        F::Error: Debug + Display,
    {
        let filter = filter.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_getLogs", vec![filter]).await
    }

    /// Returns an EIP-191 signature over the provided data
    pub async fn eth_sign<A, M>(&mut self, address: A, message: M) -> RPCResult<Signature>
    where
        A: TryInto<Address>,
        A::Error: Debug + Display,
        M: TryInto<Bytecode>,
        M::Error: Debug + Display,
    {
        let address = address.try_into().map_err(jsonrpc_rs::map_error)?;
        let message = message.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_sign", (address, message)).await
    }

    /// Returns an RLP encoded transaction signed by the specified account.
    pub async fn eth_sign_transaction<T>(&mut self, transaction: T) -> RPCResult<Bytecode>
    where
        T: TryInto<Transaction>,
        T::Error: Debug + Display,
    {
        let transaction = transaction.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_signTransaction", vec![transaction])
            .await
    }

    /// Returns the balance of the account given address.
    pub async fn eth_get_balance<A>(&mut self, address: A) -> RPCResult<U256>
    where
        A: TryInto<Address>,
        A::Error: Debug + Display,
    {
        let address = address.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_getBalance", vec![address]).await
    }

    /// Returns the number of transactions sent from an address
    pub async fn eth_get_transaction_count<A>(&mut self, address: A) -> RPCResult<U256>
    where
        A: TryInto<Address>,
        A::Error: Debug + Display,
    {
        let address = address.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getTransactionCount", vec![address])
            .await
    }

    /// Submit a raw transaction.
    pub async fn eth_send_raw_transaction<B>(&mut self, raw: B) -> RPCResult<H256>
    where
        B: TryInto<Bytecode>,
        B::Error: Debug + Display,
    {
        let raw = raw.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_sendRawTransaction", vec![raw])
            .await
    }

    pub async fn eth_get_transaction_by_hash<H>(
        &mut self,
        tx_hash: H,
    ) -> RPCResult<Option<Transaction>>
    where
        H: TryInto<H256>,
        H::Error: Debug + Display,
    {
        let tx_hash = tx_hash.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getTransactionByHash", vec![tx_hash])
            .await
    }

    /// Returns the receipt of a transaction by transaction hash
    pub async fn eth_get_transaction_receipt<H>(
        &mut self,
        tx_hash: H,
    ) -> RPCResult<Option<TransactionReceipt>>
    where
        H: TryInto<H256>,
        H::Error: Debug + Display,
    {
        let tx_hash = tx_hash.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getTransactionReceipt", vec![tx_hash])
            .await
    }
}

#[cfg(test)]
mod tests {

    use ethers_types_rs::SyncingStatus;
    use jsonrpc_rs::RPCResult;
    use serde_json::json;

    use crate::providers::http;

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

            assert_eq!(block.transactions.len(), tx_count.as_usize());

            let uncles = provider
                .eth_get_uncle_count_by_block_hash(hash.clone())
                .await?;

            assert_eq!(block.uncles.len(), uncles.as_usize());

            if let Some(number) = block.number {
                let uncles = provider.eth_get_uncle_count_by_block_number(number).await?;

                assert_eq!(block.uncles.len(), uncles.as_usize());
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

    #[async_std::test]
    async fn test_eth_sign() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let sig = provider
            .eth_sign(
                "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
                "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
            )
            .await
            .expect("eth_new_filter");

        log::debug!("eth_getFilterChanges {:?}", sig);
    }

    #[async_std::test]
    async fn test_eth_sign_transaction() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:8545");

        let balance = provider
            .eth_get_balance("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")
            .await
            .expect("eth_getBlance");

        log::debug!("balance {}", balance);

        let nonce = provider
            .eth_get_transaction_count("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720")
            .await
            .expect("eth_getTransactionCount");

        log::debug!("nonce {}", nonce);

        provider
            .eth_sign_transaction(json!({
                "nonce": nonce,
                "from": "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
                "to": "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
                "value":"0x11",
                "gas": "0x11",
                "input":"0x0"
            }))
            .await
            .expect_err("eth_signTransaction not support");
    }

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
