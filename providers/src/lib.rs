use std::fmt::{Debug, Display};

use ethers_utils_rs::{
    eth::{Address, BlockHash, Data, Number},
    hex,
};
use jsonrpc_rs::RPCResult;
use types::{Block, BlockNumberOrTag, SyncingStatus, Transaction};

pub mod error;
pub mod providers;
pub mod types;

/// Ether network api provider
///
#[derive(Clone)]
#[allow(dead_code)]
pub struct Provider {
    rpc_client: jsonrpc_rs::Client,
}

impl Provider {
    pub(crate) fn new(rpc_client: jsonrpc_rs::Client) -> Self {
        Self { rpc_client }
    }

    /// Get provider inner jsonrpc [`Client`](jsonrpc_rs::Client) ref.
    pub fn client(&mut self) -> &mut jsonrpc_rs::Client {
        &mut self.rpc_client
    }

    /// Returns the number of most recent block.
    pub async fn eth_block_number(&mut self) -> RPCResult<u64> {
        let block_number: String = self
            .rpc_client
            .call("eth_blockNumber", Vec::<String>::new())
            .await?;

        Ok(hex::hex_to_integer(&block_number).map_err(jsonrpc_rs::map_error)?)
    }

    /// Returns the chain ID of the current network
    pub async fn eth_chain_id(&mut self) -> RPCResult<u64> {
        let block_number: String = self
            .rpc_client
            .call("eth_chainId", Vec::<String>::new())
            .await?;

        Ok(hex::hex_to_integer(&block_number).map_err(jsonrpc_rs::map_error)?)
    }

    /// Returns information about a block by hash.
    pub async fn eth_get_block_by_hash<B>(
        &mut self,
        block_hash: B,
        hydrated: bool,
    ) -> RPCResult<Block>
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
    ) -> RPCResult<Block>
    where
        BT: TryInto<BlockNumberOrTag>,
        BT::Error: Debug + Display,
    {
        let block_number_or_tag = block_number_or_tag
            .try_into()
            .map_err(jsonrpc_rs::map_error)?;

        self.rpc_client
            .call("eth_getBlockByHash", (block_number_or_tag, hydrated))
            .await
    }

    /// Returns the number of transactions in a block from a block matching the given block hash
    pub async fn eth_get_block_transaction_count_by_hash<H>(&mut self, hash: H) -> RPCResult<u64>
    where
        H: TryInto<BlockHash>,
        H::Error: Debug + Display,
    {
        let hash = hash.try_into().map_err(jsonrpc_rs::map_error)?;

        let count: String = self
            .rpc_client
            .call("eth_getBlockTransactionCountByNumber", vec![hash])
            .await?;

        Ok(hex::hex_to_integer(&count).map_err(jsonrpc_rs::map_error)?)
    }

    /// Returns the number of uncles in a block from a block matching the given block hash
    pub async fn eth_get_uncle_count_by_block_hash<H>(&mut self, hash: H) -> RPCResult<u64>
    where
        H: TryInto<BlockHash>,
        H::Error: Debug + Display,
    {
        let hash = hash.try_into().map_err(jsonrpc_rs::map_error)?;

        let count: String = self
            .rpc_client
            .call("eth_getUncleCountByBlockHash", vec![hash])
            .await?;

        Ok(hex::hex_to_integer(&count).map_err(jsonrpc_rs::map_error)?)
    }

    /// Returns the number of uncles in a block from a block matching the given block hash
    pub async fn eth_get_uncle_count_by_block_number<BT>(
        &mut self,
        block_number_or_tag: BT,
    ) -> RPCResult<u64>
    where
        BT: TryInto<BlockNumberOrTag>,
        BT::Error: Debug + Display,
    {
        let block_number_or_tag = block_number_or_tag
            .try_into()
            .map_err(jsonrpc_rs::map_error)?;

        let count: String = self
            .rpc_client
            .call("eth_getUncleCountByBlockNumber", vec![block_number_or_tag])
            .await?;

        Ok(hex::hex_to_integer(&count).map_err(jsonrpc_rs::map_error)?)
    }

    /// Returns an object with data about the sync status or false
    pub async fn eth_syncing(&mut self) -> RPCResult<SyncingStatus> {
        self.rpc_client.call("eth_syncing", ()).await
    }

    /// Returns the client coinbase address.
    pub async fn eth_coinbase(&mut self) -> RPCResult<Address> {
        self.rpc_client.call("eth_coinbase", ()).await
    }

    /// Returns a list of addresses owned by client.
    pub async fn eth_accounts(&mut self) -> RPCResult<Vec<Address>> {
        self.rpc_client.call("eth_accounts", ()).await
    }

    /// Executes a new message call immediately without creating a transaction on the block chain.
    pub async fn eth_call<TX, BT>(
        &mut self,
        transaction: TX,
        block_number_or_tag: Option<BT>,
    ) -> RPCResult<Data>
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
    ) -> RPCResult<Number>
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
                .call("eth_estimateGas", (transaction, block_number_or_tag))
                .await
        } else {
            self.rpc_client
                .call("eth_estimateGas", vec![transaction])
                .await
        }
    }
}

#[cfg(test)]
mod tests {

    use jsonrpc_rs::RPCResult;

    use crate::{providers::http, types::SyncingStatus};

    #[async_std::test]
    async fn test_block_number() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

        let block_number = provider.eth_block_number().await?;

        log::debug!("block number {}", block_number);

        Ok(())
    }

    #[async_std::test]
    async fn test_chain_id() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

        let block_number = provider.eth_chain_id().await?;

        log::debug!("chain_id {}", block_number);

        Ok(())
    }

    #[async_std::test]
    async fn test_eth_getblockbynumber() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

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

        let mut provider = http::connect_to("http://localhost:1545");

        let block = provider.eth_get_block_by_number("latest", true).await?;

        let tx_count = provider
            .eth_get_block_transaction_count_by_hash(block.hash.clone())
            .await?;

        assert_eq!(block.transactions.len() as u64, tx_count);

        let uncles = provider
            .eth_get_uncle_count_by_block_hash(block.hash.clone())
            .await?;

        assert_eq!(block.uncles.len() as u64, uncles);

        let uncles = provider
            .eth_get_uncle_count_by_block_number(block.number)
            .await?;

        assert_eq!(block.uncles.len() as u64, uncles);

        Ok(())
    }

    #[async_std::test]
    async fn test_eth_syncing() -> RPCResult<()> {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

        let status = provider.eth_syncing().await?;

        assert_eq!(status, SyncingStatus::False);

        Ok(())
    }

    #[async_std::test]
    async fn test_eth_coinbase() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

        let address = provider.eth_coinbase().await.expect("eth_coinbase");

        // log::debug!("{}", status);

        assert_eq!(
            address.to_string(),
            "0xe59d4e3dc9e21f009dc88c1ba1ecffbdfe123886"
        );
    }

    #[async_std::test]
    async fn test_eth_accounts() {
        _ = pretty_env_logger::try_init();

        let mut provider = http::connect_to("http://localhost:1545");

        let accounts = provider.eth_accounts().await.expect("eth_coinbase");

        log::debug!(
            "{:?}",
            accounts.iter().map(|a| a.to_string()).collect::<Vec<_>>()
        );
    }
}
