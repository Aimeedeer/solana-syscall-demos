use anyhow::Result;
use futures_util::StreamExt;
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
    signature::Signature,
    signature::Signer,
    system_program, system_transaction,
    sysvar::rent::Rent,
    transaction::Transaction,
};
use solana_transaction_status::{TransactionDetails, UiTransactionEncoding};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::unbounded_channel;
use crate::util::Config;

/// Demo all async `PubsubClient` subscriptions.
///
/// This creates a tokio runtime,
/// spawns a task for every subscription type,
/// which subscribes and sends back a ready message and an unsubscribe channel (closure),
/// then loops on printing messages.
/// The main task then waits for user input before unsubscribing and waiting on the tasks.
pub fn demo_pubsub_client_async(
    config: &Config,
    rpc_client: RpcClient,
) -> Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(async move {
        let mut stdin = tokio::io::stdin();

        println!("press any key to begin, then press another key to end");
        stdin.read_u8().await?;

        // Subscription tasks will send a ready signal when they have subscribed.
        let (ready_sender, mut ready_receiver) = unbounded_channel::<()>();

        // Channel to receive unsubscribe channels (actually closures).
        // These receive a pair of `(Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>), &'static str)`,
        // where the first is a closure to call to unsubsribe, the second is the subscription name.
        let (unsubscribe_sender, mut unsubscribe_receiver) = unbounded_channel::<(_, &'static str)>();

        // Test transactions for signature subscriptions.
        // We'll send these after all the subscriptions are running.
        let transfer_amount = Rent::default().minimum_balance(0);
        let recent_blockhash = rpc_client.get_latest_blockhash()?;
        let transactions: Vec<Transaction> = (0..5)
            .map(|_| {
                system_transaction::transfer(
                    &config.keypair,
                    &solana_sdk::pubkey::new_rand(),
                    transfer_amount,
                    recent_blockhash,
                )
            })
            .collect();
        let signatures: HashSet<Signature> =
            transactions.iter().map(|tx| tx.signatures[0]).collect();

        let config_pubkey = config.keypair.pubkey();

        // The `PubsubClient` must be `Arc`ed to share it across tasks.
        let pubsub_client = Arc::new(PubsubClient::new(&config.websocket_url).await?);

        let mut join_handles = vec![];

        join_handles.push(("slot", tokio::spawn({
            // Clone things we need before moving their clones into the `async move` block.
            //
            // The subscriptions have to be made from the tasks that will receive the subscription messages,
            // because the subscription streams hold a reference to the `PubsubClient`.
            // Otherwise we would just subscribe on the main task and send the receivers out to other tasks.

            let ready_sender = ready_sender.clone();
            let unsubscribe_sender = unsubscribe_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut slot_notifications, slot_unsubscribe) =
                    pubsub_client.slot_subscribe().await?;

                // Report back to the main thread and drop our senders so that the channels can close.
                ready_sender.send(()).expect("channel");
                unsubscribe_sender.send((slot_unsubscribe, "slot"))
                    .map_err(|e| format!("{}", e)).expect("channel");
                drop((ready_sender, unsubscribe_sender));

                // Do something with the subscribed messages.
                // This loop will end once the main task unsubscribes.
                while let Some(slot_info) = slot_notifications.next().await {
                    println!("------------------------------------------------------------");
                    println!("slot pubsub result: {:?}", slot_info);
                }

                // This type hint is necessary to allow the `async move` block to use `?`.
                Ok::<_, anyhow::Error>(())
            }
        })));

        join_handles.push(("slot_updates", tokio::spawn({
            let ready_sender = ready_sender.clone();
            let unsubscribe_sender = unsubscribe_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut slot_updates_notifications, slot_updates_unsubscribe) =
                    pubsub_client.slot_updates_subscribe().await?;

                ready_sender.send(()).expect("channel");
                unsubscribe_sender.send((slot_updates_unsubscribe, "slot_updates"))
                    .map_err(|e| format!("{}", e)).expect("channel");
                drop((ready_sender, unsubscribe_sender));

                while let Some(slot_updates) = slot_updates_notifications.next().await {
                    println!("------------------------------------------------------------");
                    println!("slot_updates pubsub result: {:?}", slot_updates);
                }

                Ok::<_, anyhow::Error>(())
            }
        })));

        join_handles.push(("logs", tokio::spawn({
            let ready_sender = ready_sender.clone();
            let unsubscribe_sender = unsubscribe_sender.clone();
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
                    .await?;

                ready_sender.send(()).expect("channel");
                unsubscribe_sender.send((logs_unsubscribe, "logs"))
                    .map_err(|e| format!("{}", e)).expect("channel");
                drop((ready_sender, unsubscribe_sender));


                while let Some(logs) = logs_notifications.next().await {
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

                Ok::<_, anyhow::Error>(())
            }
        })));

        join_handles.push(("root", tokio::spawn({
            let ready_sender = ready_sender.clone();
            let unsubscribe_sender = unsubscribe_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut root_notifications, root_unsubscribe) =
                    pubsub_client.root_subscribe().await?;

                ready_sender.send(()).expect("channel");
                unsubscribe_sender.send((root_unsubscribe, "root"))
                    .map_err(|e| format!("{}", e)).expect("channel");
                drop((ready_sender, unsubscribe_sender));

                while let Some(root) = root_notifications.next().await {
                    println!("------------------------------------------------------------");
                    println!("root pubsub result: {:?}", root);
                }

                Ok::<_, anyhow::Error>(())
            }
        })));

        // This subscription will fail unless the validator is started with
        // `----rpc-pubsub-enable-block-subscription`.
        join_handles.push(("block", tokio::spawn({
            let ready_sender = ready_sender.clone();
            let unsubscribe_sender = unsubscribe_sender.clone();
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
                    .await?;

                ready_sender.send(()).expect("channel");
                unsubscribe_sender.send((block_unsubscribe, "block"))
                    .map_err(|e| format!("{}", e)).expect("channel");
                drop((ready_sender, unsubscribe_sender));

                while let Some(block) = block_notifications.next().await {
                    println!("------------------------------------------------------------");
                    println!("block pubsub result: {:?}", block);
                }

                Ok::<_, anyhow::Error>(())
            }
        })));

        // don't see the result from localhost/devnet
        join_handles.push(("program", tokio::spawn({
            let ready_sender = ready_sender.clone();
            let unsubscribe_sender = unsubscribe_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut program_notifications, program_unsubscribe) = pubsub_client
                    .program_subscribe(
                        &system_program::ID,
                        Some(RpcProgramAccountsConfig {
                            ..RpcProgramAccountsConfig::default()
                        }),
                    )
                    .await?;

                ready_sender.send(()).expect("channel");
                unsubscribe_sender.send((program_unsubscribe, "program"))
                    .map_err(|e| format!("{}", e)).expect("channel");
                drop((ready_sender, unsubscribe_sender));

                while let Some(program) = program_notifications.next().await {
                    println!("------------------------------------------------------------");
                    println!("program pubsub result: {:?}", program);
                }

                Ok::<_, anyhow::Error>(())
            }
        })));

        join_handles.push(("account", tokio::spawn({
            let ready_sender = ready_sender.clone();
            let unsubscribe_sender = unsubscribe_sender.clone();
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
                    .await?;

                ready_sender.send(()).expect("channel");
                unsubscribe_sender.send((account_unsubscribe, "account"))
                    .map_err(|e| format!("{}", e)).expect("channel");
                drop((ready_sender, unsubscribe_sender));

                while let Some(account) = account_notifications.next().await {
                    println!("------------------------------------------------------------");
                    println!("account pubsub result: {:?}", account);
                }

                Ok::<_, anyhow::Error>(())
            }
        })));

        // This subscription will fail unless the validator is started with
        // `----rpc-pubsub-enable-vote-subscription`.
        join_handles.push(("vote", tokio::spawn({
            let ready_sender = ready_sender.clone();
            let unsubscribe_sender = unsubscribe_sender.clone();
            let pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut vote_notifications, vote_unsubscribe) =
                    pubsub_client.vote_subscribe().await?;

                ready_sender.send(()).expect("channel");
                unsubscribe_sender.send((vote_unsubscribe, "vote"))
                    .map_err(|e| format!("{}", e)).expect("channel");
                drop((ready_sender, unsubscribe_sender));

                while let Some(vote) = vote_notifications.next().await {
                    println!("------------------------------------------------------------");
                    println!("vote pubsub result: {:?}", vote);
                }

                Ok::<_, anyhow::Error>(())
            }
        })));

        for signature in signatures {
            join_handles.push(("signature", tokio::spawn({
                let ready_sender = ready_sender.clone();
                let unsubscribe_sender = unsubscribe_sender.clone();
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
                        .await?;

                    ready_sender.send(()).expect("channel");
                    unsubscribe_sender.send((signature_unsubscribe, "signature"))
                        .map_err(|e| format!("{}", e)).expect("channel");
                    drop((ready_sender, unsubscribe_sender));

                    while let Some(sig_response) = signature_notifications.next().await {
                        println!("------------------------------------------------------------");
                        println!("signature pubsub result: {:?}", sig_response);
                    }

                    Ok::<_, anyhow::Error>(())
                }
            })));
        }

        // Drop these senders so that their receivers return `None` below.
        drop(ready_sender);
        drop(unsubscribe_sender);

        // Wait until all subscribers are ready.
        while let Some(_) = ready_receiver.recv().await { }

        println!("sending test transactions");
        for tx in transactions {
            let sig = rpc_client.send_and_confirm_transaction(&tx)?;
            println!("transfer sig: {}", sig);
        }

        // Wait for input.
        stdin.read_u8().await?;

        // Unsubscribe from everything, which will shutdown all the tasks.
        while let Some((unsubscribe, name)) = unsubscribe_receiver.recv().await {
            println!("unsubscribing from {}", name);
            unsubscribe().await
        }

        // Wait for the tasks.
        for (name, handle) in join_handles {
            println!("waiting on task {}", name);
            if let Ok(Err(e)) = handle.await {
                println!("task {} failed: {}", name, e);
            }
        }

        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}
