use anyhow::Result;
use futures_util::StreamExt;
use solana_account_decoder::UiAccount;
use solana_client::rpc_response::{
    Response, RpcKeyedAccount, RpcLogsResponse, RpcSignatureResult, SlotInfo, SlotUpdate,
};
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{
        RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcSignatureSubscribeConfig,
        RpcTransactionLogsConfig, RpcTransactionLogsFilter,
    },
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    rpc_port,
    signature::Signature,
    signature::{Keypair, Signer},
    slot_history::Slot,
    system_program, system_transaction,
    sysvar::rent::Rent,
    transaction::Transaction,
};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task;

pub fn demo_pubsub_client_async(
    config_keypair: Keypair,
    rpc_client: RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(async move {
        // Track when subscriptions are ready
        let (ready_sender, mut ready_receiver) = unbounded_channel::<()>();

        let (slot_sender, mut slot_receiver) = unbounded_channel::<SlotInfo>();
        let (slot_updates_sender, mut slot_updates_receiver) = unbounded_channel::<SlotUpdate>();
        let (logs_sender, mut logs_receiver) = unbounded_channel::<Response<RpcLogsResponse>>();
        let (root_sender, mut root_receiver) = unbounded_channel::<Slot>();
        let (program_sender, mut program_receiver) =
            unbounded_channel::<Response<RpcKeyedAccount>>();
        let (account_sender, mut account_receiver) = unbounded_channel::<Response<UiAccount>>();
        let (signature_sender, mut signature_receiver) =
            unbounded_channel::<(Signature, Response<RpcSignatureResult>)>();

        let ws_url = &format!("ws://127.0.0.1:{}/", rpc_port::DEFAULT_RPC_PUBSUB_PORT);
        // let ws_url = "wss://api.devnet.solana.com/";

        // transactions for test
        let transfer_amount = Rent::default().minimum_balance(0);
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        let transactions: Vec<Transaction> = (0..10)
            .map(|_| {
                system_transaction::transfer(
                    &config_keypair,
                    &solana_sdk::pubkey::new_rand(),
                    transfer_amount,
                    recent_blockhash,
                )
            })
            .collect();
        let mut signatures: HashSet<Signature> =
            transactions.iter().map(|tx| tx.signatures[0]).collect();

        let config_pubkey = config_keypair.pubkey();
        let pubsub_client = Arc::new(PubsubClient::new(ws_url).await.unwrap());

        let task_slot_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut slot_notifications, slot_unsubscribe) =
                    pubsub_client.slot_subscribe().await.unwrap();

                ready_sender.send(()).unwrap();

                while let Some(slot_info) = slot_notifications.next().await {
                    slot_sender.send(slot_info).unwrap();
                }

                slot_unsubscribe().await;
            }
        });

        let task_slot_updates_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut slot_updates_notifications, slot_updates_unsubscribe) =
                    pubsub_client.slot_updates_subscribe().await.unwrap();

                ready_sender.send(()).unwrap();

                while let Some(slot_updates) = slot_updates_notifications.next().await {
                    slot_updates_sender.send(slot_updates).unwrap();
                }

                slot_updates_unsubscribe().await;
            }
        });

        let task_logs_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut logs_notifications, logs_unsubscribe) = pubsub_client
                    .logs_subscribe(
                        RpcTransactionLogsFilter::All,
                        RpcTransactionLogsConfig {
                            commitment: Some(CommitmentConfig {
                                commitment: CommitmentLevel::Confirmed,
                            }),
                        },
                    )
                    .await
                    .unwrap();

                ready_sender.send(()).unwrap();

                while let Some(logs) = logs_notifications.next().await {
                    logs_sender.send(logs).unwrap();
                }

                logs_unsubscribe().await;
            }
        });

        let task_root_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut root_notifications, root_unsubscribe) =
                    pubsub_client.root_subscribe().await.unwrap();

                ready_sender.send(()).unwrap();

                while let Some(root) = root_notifications.next().await {
                    root_sender.send(root).unwrap();
                }

                root_unsubscribe().await;
            }
        });

        let task_program_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut program_notifications, program_unsubscribe) = pubsub_client
                    .program_subscribe(
                        &system_program::ID,
                        Some(RpcProgramAccountsConfig {
                            ..RpcProgramAccountsConfig::default()
                        }),
                    )
                    .await
                    .unwrap();

                ready_sender.send(()).unwrap();

                while let Some(program) = program_notifications.next().await {
                    program_sender.send(program).unwrap();
                }
                program_unsubscribe().await;
            }
        });

        let task_account_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut account_notifications, account_unsubscribe) = pubsub_client
                    .account_subscribe(
                        &config_pubkey,
                        Some(RpcAccountInfoConfig {
                            commitment: Some(CommitmentConfig::confirmed()),
                            ..RpcAccountInfoConfig::default()
                        }),
                    )
                    .await
                    .unwrap();

                ready_sender.send(()).unwrap();

                while let Some(account) = account_notifications.next().await {
                    account_sender.send(account).unwrap();
                }
                account_unsubscribe().await;
            }
        });

        let task_signature_subscribe = tokio::spawn(async move {
            for signature in signatures {
                tokio::spawn({
                    let signature_sender = signature_sender.clone();
                    let ready_sender = ready_sender.clone();
                    let pubsub_client = Arc::clone(&pubsub_client);
                    async move {
                        let (mut signature_notifications, signature_unsubscribe) = pubsub_client
                            .signature_subscribe(
                                &signature,
                                Some(RpcSignatureSubscribeConfig {
                                    commitment: Some(CommitmentConfig::confirmed()),
                                    ..RpcSignatureSubscribeConfig::default()
                                }),
                            )
                            .await
                            .unwrap();

                        ready_sender.send(()).unwrap();

                        while let Some(sig_response) = signature_notifications.next().await {
                            signature_sender.send((signature, sig_response)).unwrap();
                        }

                        signature_unsubscribe().await;
                    }
                });
            }
        });

        let task_slot_receiver = task::spawn(async move {
            loop {
                if let Some(result) = slot_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("slot pubsub result: {:?}", result);
                }
            }
        });

        let task_slot_updates_receiver = task::spawn(async move {
            loop {
                if let Some(result) = slot_updates_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("slot_updates pubsub result: {:?}", result);
                }
            }
        });

        let task_logs_receiver = task::spawn(async move {
            loop {
                if let Some(logs) = logs_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("logs pubsub result:");

                    println!("Transaction executed in slot {}:", logs.context.slot);
                    println!("  Signature: {}", logs.value.signature);
                    println!(
                        "  Status: {}",
                        logs.value
                            .err
                            .map(|err| err.to_string())
                            .unwrap_or_else(|| "Ok".to_string())
                    );
                    println!("  Log Messages:");
                    for log in logs.value.logs {
                        println!("    {}", log);
                    }
                }
            }
        });

        let task_root_receiver = task::spawn(async move {
            loop {
                if let Some(result) = root_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("root pubsub result: {:?}", result);
                }
            }
        });

        let task_program_receiver = task::spawn(async move {
            loop {
                if let Some(result) = program_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("program pubsub result: {:?}", result);
                }
            }
        });

        let task_account_receiver = task::spawn(async move {
            loop {
                if let Some(result) = account_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("account pubsub result: {:?}", result);
                }
            }
        });

        let task_signature_receiver = task::spawn(async move {
            loop {
                if let Some(result) = signature_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("signature pubsub result: {:?}", result);
                }
            }
        });

        let task_test_tx = task::spawn(async move {
            // send testing txs when all subscriptions are ready
            // signals from slot, slot_updates, logs, root, program, account subscriptions
            ready_receiver.recv().await;
            ready_receiver.recv().await;
            ready_receiver.recv().await;
            ready_receiver.recv().await;
            ready_receiver.recv().await;
            ready_receiver.recv().await;
            ready_receiver.recv().await;

            // signals from 10 test signature subscriptions
            for i in 0..10 {
                ready_receiver.recv().await;
            }

            println!("sending out testing transaction");
            for tx in transactions {
                let sig = rpc_client.send_and_confirm_transaction(&tx).unwrap();
                println!("transfer sig: {}", sig);
            }
        });

        task_slot_subscribe.await;
        task_slot_receiver.await;

        task_slot_updates_subscribe.await;
        task_slot_updates_receiver.await;

        task_logs_subscribe.await;
        task_logs_receiver.await;

        task_root_subscribe.await;
        task_root_receiver.await;

        task_program_subscribe.await;
        task_program_receiver.await;

        task_account_subscribe.await;
        task_account_receiver.await;

        task_signature_subscribe.await;
        task_signature_receiver.await;

        task_test_tx.await;
    });

    Ok(())
}
