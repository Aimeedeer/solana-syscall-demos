use anyhow::Result;
use futures_util::StreamExt;
use solana_account_decoder::UiAccount;
use solana_client::rpc_response::{
    Response, RpcBlockUpdate, RpcKeyedAccount, RpcLogsResponse, RpcSignatureResult, RpcVote,
    SlotInfo, SlotUpdate,
};
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
    rpc_port,
    signature::Signature,
    signature::{Keypair, Signer},
    slot_history::Slot,
    system_program, system_transaction,
    sysvar::rent::Rent,
    transaction::Transaction,
};
use solana_transaction_status::{TransactionDetails, UiTransactionEncoding};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, unbounded_channel};
use tokio::sync::oneshot;
use tokio::task;
use tokio::time::sleep;
use tokio::time::Duration;

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
        let (block_sender, mut block_receiver) = unbounded_channel::<Response<RpcBlockUpdate>>();
        let (program_sender, mut program_receiver) =
            unbounded_channel::<Response<RpcKeyedAccount>>();
        let (account_sender, mut account_receiver) = unbounded_channel::<Response<UiAccount>>();
        let (vote_sender, mut vote_receiver) = unbounded_channel::<RpcVote>();
        let (signature_sender, mut signature_receiver) =
            unbounded_channel::<(Signature, Response<RpcSignatureResult>)>();

        // channels for unsubscribe
        let (slot_unsubscribe_sender, mut slot_unsubscribe_receiver) = oneshot::channel();
        let (slot_updates_unsubscribe_sender, mut slot_updates_unsubscribe_receiver) =
            oneshot::channel();
        let (logs_unsubscribe_sender, mut logs_unsubscribe_receiver) = oneshot::channel();
        let (root_unsubscribe_sender, mut root_unsubscribe_receiver) = oneshot::channel();
        let (block_unsubscribe_sender, mut block_unsubscribe_receiver) = oneshot::channel();
        let (program_unsubscribe_sender, mut program_unsubscribe_receiver) = oneshot::channel();
        let (account_unsubscribe_sender, mut account_unsubscribe_receiver) = oneshot::channel();
        let (vote_unsubscribe_sender, mut vote_unsubscribe_receiver) = oneshot::channel();
        let (signature_unsubscribe_sender, mut signature_unsubscribe_receiver) = channel(5);

        // transactions for test
        let transfer_amount = Rent::default().minimum_balance(0);
        let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
        let transactions: Vec<Transaction> = (0..5)
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

        // let ws_url = "wss://api.devnet.solana.com/";
        let ws_url = &format!("ws://127.0.0.1:{}/", rpc_port::DEFAULT_RPC_PUBSUB_PORT);
        let pubsub_client = Arc::new(PubsubClient::new(ws_url).await.unwrap());

        let task_slot_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut slot_notifications, slot_unsubscribe) =
                    pubsub_client.slot_subscribe().await.unwrap();

                ready_sender.send(()).unwrap();

                if let Err(_) = slot_unsubscribe_sender.send(slot_unsubscribe) {
                    println!("slot_unsubscribe receiver dropped");
                }

                while let Some(slot_info) = slot_notifications.next().await {
                    slot_sender.send(slot_info).unwrap();
                }
            }
        });

        let task_slot_updates_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut slot_updates_notifications, slot_updates_unsubscribe) =
                    pubsub_client.slot_updates_subscribe().await.unwrap();

                ready_sender.send(()).unwrap();
                if let Err(_) = slot_updates_unsubscribe_sender.send(slot_updates_unsubscribe) {
                    println!("slot_updates_unsubscribe receiver dropped");
                }

                while let Some(slot_updates) = slot_updates_notifications.next().await {
                    slot_updates_sender.send(slot_updates).unwrap();
                }
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
                if let Err(_) = logs_unsubscribe_sender.send(logs_unsubscribe) {
                    println!("logs_unsubscribe receiver dropped");
                }

                while let Some(logs) = logs_notifications.next().await {
                    logs_sender.send(logs).unwrap();
                }
            }
        });

        let task_root_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut root_notifications, root_unsubscribe) =
                    pubsub_client.root_subscribe().await.unwrap();

                ready_sender.send(()).unwrap();
                if let Err(_) = root_unsubscribe_sender.send(root_unsubscribe) {
                    println!("root_unsubscribe receiver dropped");
                }

                while let Some(root) = root_notifications.next().await {
                    root_sender.send(root).unwrap();
                }
            }
        });

        // thread 'tokio-runtime-worker' panicked at 'called
        // `Result::unwrap()` on an `Err` value: SubscribeFailed {
        // reason: "Method not found (-32601)", message:
        // "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32601,\"message\":\"Method
        // not found\"},\"id\":8}" }',
        let task_block_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut block_notifications, block_unsubscribe) = pubsub_client
                    .block_subscribe(
                        RpcBlockSubscribeFilter::All,
                        Some(RpcBlockSubscribeConfig {
                            commitment: Some(CommitmentConfig {
                                commitment: CommitmentLevel::Processed, // Confirmed, Finalized
                            }),
                            encoding: Some(UiTransactionEncoding::Json),
                            transaction_details: Some(TransactionDetails::Signatures),
                            show_rewards: None,
                            max_supported_transaction_version: None,
                        }),
                    )
                    .await
                    .unwrap();

                ready_sender.send(()).unwrap();
                if let Err(_) = block_unsubscribe_sender.send(block_unsubscribe) {
                    println!("block_unsubscribe receiver dropped");
                }

                while let Some(block) = block_notifications.next().await {
                    block_sender.send(block).unwrap();
                }
            }
        });

        // don't see the result from localhost/devnet
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
                if let Err(_) = program_unsubscribe_sender.send(program_unsubscribe) {
                    println!("program_unsubscribe receiver dropped");
                }

                while let Some(program) = program_notifications.next().await {
                    program_sender.send(program).unwrap();
                }
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

                if let Err(_) = account_unsubscribe_sender.send(account_unsubscribe) {
                    println!("account_unsubscribe receiver dropped");
                }

                while let Some(account) = account_notifications.next().await {
                    account_sender.send(account).unwrap();
                }
            }
        });

        // Thread 'Tokio-runtime-worker' panicked at 'called
        // `Result::unwrap()` on an `Err` value: SubscribeFailed {
        // reason: "Method not found (-32601)", message:
        // "{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32601,\"message\":\"Method
        // not found\"},\"id\":1}" }'
        let task_vote_subscribe = tokio::spawn({
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut vote_notifications, vote_unsubscribe) =
                    pubsub_client.vote_subscribe().await.unwrap();

                ready_sender.send(()).unwrap();

                if let Err(_) = vote_unsubscribe_sender.send(vote_unsubscribe) {
                    println!("vote_unsubscribe receiver dropped");
                }

                while let Some(vote) = vote_notifications.next().await {
                    vote_sender.send(vote).unwrap();
                }
            }
        });

        let task_signature_subscribe = tokio::spawn(async move {
            let ready_sender = ready_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);

            for signature in signatures {
                tokio::spawn({
                    let signature_sender = signature_sender.clone();
                    let ready_sender = ready_sender.clone();
                    let pubsub_client = Arc::clone(&pubsub_client);
                    let signature_unsubscribe_sender = signature_unsubscribe_sender.clone();
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
                        if let Err(_) = signature_unsubscribe_sender
                            .send(signature_unsubscribe)
                            .await
                        {
                            println!("signature_unsubscribe receiver dropped");
                        }

                        while let Some(sig_response) = signature_notifications.next().await {
                            signature_sender.send((signature, sig_response)).unwrap();
                        }
                    }
                });
            }
        });

        let task_slot_receiver = task::spawn(async move {
            loop {
                if let Some(result) = slot_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("slot pubsub result: {:?}", result);
                } else {
                    break;
                }
            }
        });

        let task_slot_updates_receiver = task::spawn(async move {
            loop {
                if let Some(result) = slot_updates_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("slot_updates pubsub result: {:?}", result);
                } else {
                    break;
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
                } else {
                    break;
                }
            }
        });

        let task_root_receiver = task::spawn(async move {
            loop {
                if let Some(result) = root_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("root pubsub result: {:?}", result);
                } else {
                    break;
                }
            }
        });

        let task_block_receiver = task::spawn(async move {
            loop {
                if let Some(result) = block_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("block pubsub result: {:?}", result);
                } else {
                    break;
                }
            }
        });

        let task_program_receiver = task::spawn(async move {
            loop {
                if let Some(result) = program_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("program pubsub result: {:?}", result);
                } else {
                    break;
                }
            }
        });

        let task_account_receiver = task::spawn(async move {
            loop {
                if let Some(result) = account_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("account pubsub result: {:?}", result);
                } else {
                    break;
                }
            }
        });

        let task_vote_receiver = task::spawn(async move {
            loop {
                if let Some(result) = vote_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("vote pubsub result: {:?}", result);
                } else {
                    break;
                }
            }
        });

        let task_signature_receiver = task::spawn(async move {
            loop {
                if let Some(result) = signature_receiver.recv().await {
                    println!("------------------------------------------------------------");
                    println!("signature pubsub result: {:?}", result);
                } else {
                    break;
                }
            }
        });

        // send testing txs when all subscriptions are ready
        let task_test_tx = task::spawn(async move {
            // signals from slot, slot_updates, logs, root, block, program, account, vote subscriptions
            ready_receiver.recv().await;
            ready_receiver.recv().await;

            ready_receiver.recv().await;
            ready_receiver.recv().await;

            ready_receiver.recv().await;
            ready_receiver.recv().await;

            ready_receiver.recv().await;
            ready_receiver.recv().await;

            // signals from 5 test signature subscriptions
            for i in 0..5 {
                ready_receiver.recv().await;
            }

            println!("sending out testing transaction");
            for tx in transactions {
                let sig = rpc_client.send_and_confirm_transaction(&tx).unwrap();
                println!("transfer sig: {}", sig);
            }
        });

        // unsubscribe all
        match slot_unsubscribe_receiver.await {
            Ok(slot_unsubscribe) => {
                sleep(Duration::from_secs(5)).await;
                slot_unsubscribe().await;
            }
            Err(e) => {
                println!("slot_unsubscribe_receiver error: {}", e);
            }
        }

        match slot_updates_unsubscribe_receiver.await {
            Ok(slot_updates_unsubscribe) => {
                sleep(Duration::from_secs(5)).await;
                slot_updates_unsubscribe().await;
            }
            Err(e) => {
                println!("slot_updates_unsubscribe_receiver error: {}", e);
            }
        }

        match logs_unsubscribe_receiver.await {
            Ok(logs_unsubscribe) => {
                sleep(Duration::from_secs(5)).await;
                logs_unsubscribe().await;
            }
            Err(e) => {
                println!("logs_unsubscribe_receiver error: {}", e);
            }
        }

        match root_unsubscribe_receiver.await {
            Ok(root_unsubscribe) => {
                sleep(Duration::from_secs(5)).await;
                root_unsubscribe().await;
            }
            Err(e) => {
                println!("root_unsubscribe_receiver error: {}", e);
            }
        }

        match block_unsubscribe_receiver.await {
            Ok(block_unsubscribe) => {
                sleep(Duration::from_secs(5)).await;
                block_unsubscribe().await;
            }
            Err(e) => {
                println!("block_unsubscribe_receiver error: {}", e);
            }
        }

        match program_unsubscribe_receiver.await {
            Ok(program_unsubscribe) => {
                sleep(Duration::from_secs(5)).await;
                program_unsubscribe().await;
            }
            Err(e) => {
                println!("program_unsubscribe_receiver error: {}", e);
            }
        }

        match account_unsubscribe_receiver.await {
            Ok(account_unsubscribe) => {
                sleep(Duration::from_secs(5)).await;
                account_unsubscribe().await;
            }
            Err(e) => {
                println!("account_unsubscribe_receiver error: {}", e);
            }
        }

        match vote_unsubscribe_receiver.await {
            Ok(vote_unsubscribe) => {
                sleep(Duration::from_secs(5)).await;
                vote_unsubscribe().await;
            }
            Err(e) => {
                println!("vote_unsubscribe_receiver error: {}", e);
            }
        }

        while let Some(signature_unsubscribe) = signature_unsubscribe_receiver.recv().await {
            sleep(Duration::from_secs(1)).await;
            signature_unsubscribe().await;
        }

        task_slot_subscribe.await;
        task_slot_receiver.await;

        task_slot_updates_subscribe.await;
        task_slot_updates_receiver.await;

        task_logs_subscribe.await;
        task_logs_receiver.await;

        task_root_subscribe.await;
        task_root_receiver.await;

        task_block_subscribe.await;
        task_block_receiver.await;

        task_program_subscribe.await;
        task_program_receiver.await;

        task_account_subscribe.await;
        task_account_receiver.await;

        task_vote_subscribe.await;
        task_vote_receiver.await;

        task_signature_subscribe.await;
        task_signature_receiver.await;

        task_test_tx.await;
    });

    Ok(())
}
