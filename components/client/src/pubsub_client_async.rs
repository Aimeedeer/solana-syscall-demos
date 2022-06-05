use anyhow::Result;
use crossbeam_channel::unbounded;
use futures_util::StreamExt;
use solana_account_decoder::UiAccount;
use solana_client::rpc_response::Response;
use solana_client::rpc_response::SlotInfo;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{
        RpcAccountInfoConfig, RpcBlockSubscribeConfig, RpcBlockSubscribeFilter,
        RpcProgramAccountsConfig, RpcSignatureSubscribeConfig, RpcTransactionLogsConfig,
        RpcTransactionLogsFilter,
    },
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    hash::Hash,
    pubkey::Pubkey,
    rpc_port,
    signature::{Keypair, Signer},
    system_program, system_transaction,
};
use solana_transaction_status::{
    BlockEncodingOptions, ConfirmedBlock, TransactionDetails, UiTransactionEncoding,
    VersionedConfirmedBlock,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

pub fn demo_pubsub_client_async(
    config: &crate::util::Config,
    rpc_client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let rt = Runtime::new()?;

    println!("-------------------- pubsub client async --------------------");
    rt.spawn(async move {
        // let ws_url = &format!("ws://127.0.0.1:{}/", rpc_port::DEFAULT_RPC_PUBSUB_PORT);
        let ws_url = "wss://api.devnet.solana.com/";

        let pubsub_client = Arc::new(PubsubClient::new(ws_url).await.unwrap());

        tokio::spawn({
            let _pubsub_client = Arc::clone(&pubsub_client);

            async move {
                let (mut slot_notifications, slot_unsubscribe) =
                    _pubsub_client.slot_subscribe().await.unwrap();
                let response = slot_notifications.next().await.unwrap();
                println!("response: {:?}", response);

                slot_unsubscribe().await;
            }
        });
    });

    println!("-------------------- pubsub client async end --------------------");

    Ok(())
}
