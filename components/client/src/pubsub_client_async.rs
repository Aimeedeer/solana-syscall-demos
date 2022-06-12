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
    sysvar::rent::Rent,
    transaction::Transaction,
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
    // Create transaction for pubsub test
    let transfer_amount = Rent::default().minimum_balance(0);
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transactions: Vec<Transaction> = (0..10)
        .map(|_| {
            system_transaction::transfer(
                &config.keypair,
                &solana_sdk::pubkey::new_rand(),
                transfer_amount,
                recent_blockhash,
            )
        })
        .collect();

    let (account_sender, account_receiver) = unbounded::<Response<UiAccount>>();
    let (slot_sender, slot_receiver) = unbounded::<SlotInfo>();

    let rt = Runtime::new()?;
    let config_pubkey = config.keypair.pubkey();
    rt.spawn(async move {
        let ws_url = &format!("ws://127.0.0.1:{}/", rpc_port::DEFAULT_RPC_PUBSUB_PORT);
        // let ws_url = "wss://api.devnet.solana.com/";

        let pubsub_client = Arc::new(PubsubClient::new(ws_url).await.unwrap());
        tokio::spawn({
            let _pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut account_notifications, account_unsubscribe) = _pubsub_client
                    .account_subscribe(
                        &config_pubkey,
                        Some(RpcAccountInfoConfig {
                            commitment: Some(CommitmentConfig::confirmed()),
                            ..RpcAccountInfoConfig::default()
                        }),
                    )
                    .await
                    .unwrap();

                while let Some(account) = account_notifications.next().await {
                    account_sender.send(account).unwrap();
                }
                account_unsubscribe().await;
            }
        });

        tokio::spawn({
            let _pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut slot_notifications, slot_unsubscribe) =
                    _pubsub_client.slot_subscribe().await.unwrap();
                while let Some(slot_info) = slot_notifications.next().await {
                    slot_sender.send(slot_info).unwrap();
                }
                slot_unsubscribe().await;
            }
        });
    });

    println!("-------------------- pubsub client async receiver --------------------");
    thread::spawn(move || loop {
        match account_receiver.recv() {
            Ok(result) => {
                println!("account pubsub result: {:?}", result);
            }
            Err(e) => {
                println!("account pubsub error: {:?}", e);
                break;
            }
        }
    });

    thread::spawn(move || loop {
        match slot_receiver.recv() {
            Ok(result) => {
                println!("slot pubsub result: {:?}", result);
            }
            Err(e) => {
                println!("slot pubsub error: {:?}", e);
                break;
            }
        }
    });

    Ok(())
}
