use super::Provider;

use std::fmt::{Debug, Display};

use ethers_txrequest::TypedTransactionRequest;
use jsonrpc_rs::RPCResult;

use crate::types::*;
use ethers_primitives::*;

impl Provider {
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
        B: TryInto<H256>,
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
        H: TryInto<H256>,
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
        H: TryInto<H256>,
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
    ) -> RPCResult<Bytes>
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

    pub async fn eth_get_filter_changes<N>(&mut self, id: N) -> RPCResult<Option<FilterEvents>>
    where
        N: TryInto<U256>,
        N::Error: Debug + Display,
    {
        let id = id.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_getFilterChanges", vec![id]).await
    }

    /// Returns any arrays of all logs matching filter with given id
    pub async fn eth_get_filter_logs<N>(&mut self, id: N) -> RPCResult<FilterEvents>
    where
        N: TryInto<U256>,
        N::Error: Debug + Display,
    {
        let id = id.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_getFilterLogs", vec![id]).await
    }

    /// Returns an array of all logs matching filter with filter description
    pub async fn eth_get_logs<F>(&mut self, filter: F) -> RPCResult<FilterEvents>
    where
        F: TryInto<Filter>,
        F::Error: Debug + Display,
    {
        let filter = filter.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_getLogs", vec![filter]).await
    }

    /// Returns an EIP-191 signature over the provided data
    pub async fn eth_sign<A, M>(&mut self, address: A, message: M) -> RPCResult<Eip1559Signature>
    where
        A: TryInto<Address>,
        A::Error: Debug + Display,
        M: TryInto<Bytes>,
        M::Error: Debug + Display,
    {
        let address = address.try_into().map_err(jsonrpc_rs::map_error)?;
        let message = message.try_into().map_err(jsonrpc_rs::map_error)?;

        self.rpc_client.call("eth_sign", (address, message)).await
    }

    /// Returns an RLP encoded transaction signed by the specified account.
    pub async fn eth_sign_transaction<T>(&mut self, transaction: T) -> RPCResult<Bytes>
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
        B: TryInto<Bytes>,
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
