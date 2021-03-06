#![allow(unused)]

use anyhow::Result;
use crossbeam_channel::unbounded;
use solana_client::rpc_response::SlotInfo;
use solana_client::{
    pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{
        RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcSignatureSubscribeConfig,
        RpcTransactionLogsConfig, RpcTransactionLogsFilter,
    },
};
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    rent::Rent,
    rpc_port,
    signature::{Keypair, Signer},
    system_program, system_transaction,
};
use std::thread;

pub fn demo_pubsub_client(
    config: &crate::util::Config,
    rpc_client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    //    let ws_url = "wss://api.devnet.solana.com/";
    let ws_url = &format!("ws://127.0.0.1:{}/", rpc_port::DEFAULT_RPC_PUBSUB_PORT);

    println!("-------------------- account subscription --------------------");
    let rpc_config = Some(RpcAccountInfoConfig {
        commitment: Some(CommitmentConfig::confirmed()),
        encoding: None,
        data_slice: None,
    });

    let (mut account_subscription_client, account_subscription_receiver) =
        PubsubClient::account_subscribe(ws_url, &config.keypair.pubkey(), rpc_config.clone())?;

    let alice = Keypair::new();
    let (mut account_subscription_client_for_alice, account_subscription_receiver_for_alice) =
        PubsubClient::account_subscribe(ws_url, &alice.pubkey(), rpc_config)?;

    // send a tx for testing
    let blockhash = rpc_client.get_latest_blockhash()?;
    let transfer_amount = Rent::default().minimum_balance(0);
    let tx =
        system_transaction::transfer(&config.keypair, &alice.pubkey(), transfer_amount, blockhash);
    let sig = rpc_client.send_and_confirm_transaction(&tx)?;
    println!("transfer sig: {}", sig);

    thread::spawn(move || loop {
        match account_subscription_receiver.recv() {
            Ok(response) => {
                println!("account subscription response: {:?}", response);
            }
            Err(e) => {
                println!("account subscription error: {:?}", e);
                break;
            }
        }
    });

    thread::spawn(move || loop {
        match account_subscription_receiver_for_alice.recv() {
            Ok(response) => {
                println!("account subscription for alice response: {:?}", response);
            }
            Err(e) => {
                println!("account subscription for alice error: {:?}", e);
                break;
            }
        }
    });

    println!("-------------------- logs subscription --------------------");
    let (mut logs_subscription_client, logs_subscription_receiver) = PubsubClient::logs_subscribe(
        ws_url,
        RpcTransactionLogsFilter::All,
        RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
        },
    )?;

    thread::spawn(move || loop {
        match logs_subscription_receiver.recv() {
            Ok(logs) => {
                println!("---------- logs subscription result ----------");
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
            Err(e) => {
                println!("log subscription error: {:?}", e);
                break;
            }
        }
    });

    println!("-------------------- program subscription --------------------");
    let (mut program_subscription_client, program_subscription_receiver) =
        PubsubClient::program_subscribe(
            ws_url,
            &system_program::ID,
            Some(RpcProgramAccountsConfig {
                ..RpcProgramAccountsConfig::default()
            }),
        )?;

    thread::spawn(move || loop {
        match program_subscription_receiver.recv() {
            Ok(response) => {
                println!("program subscription response: {:?}", response);
            }
            Err(e) => {
                println!("program subscription error: {:?}", e);
                break;
            }
        }
    });

    println!("-------------------- root subscription --------------------");
    let (mut root_subscription_client, root_subscription_receiver) =
        PubsubClient::root_subscribe(ws_url)?;

    thread::spawn(move || loop {
        match root_subscription_receiver.recv() {
            Ok(result) => {
                println!("root subscription result: {:?}", result);
            }
            Err(e) => {
                println!("root subscription error: {:?}", e);
                break;
            }
        }
    });

    println!("-------------------- signature subscription --------------------");
    let alice = Keypair::new();
    let blockhash = rpc_client.get_latest_blockhash()?;
    let transfer_amount = Rent::default().minimum_balance(0);
    let tx =
        system_transaction::transfer(&config.keypair, &alice.pubkey(), transfer_amount, blockhash);
    let (mut sig_subscription_client, sig_subscription_receiver) =
        PubsubClient::signature_subscribe(
            ws_url,
            &tx.signatures[0],
            Some(RpcSignatureSubscribeConfig {
                commitment: Some(CommitmentConfig::processed()),
                enable_received_notification: Some(true),
            }),
        )?;

    let sig = rpc_client.send_and_confirm_transaction(&tx)?;
    println!("subscribe to signature: {:?}", sig);

    thread::spawn(move || loop {
        match sig_subscription_receiver.recv() {
            Ok(response) => {
                println!("signature subscription response: {:?}", response);
            }
            Err(e) => {
                println!("signature subscription error: {:?}", e);
                break;
            }
        }
    });

    println!("-------------------- slot subscription --------------------");
    let (mut slot_subscription_client, slot_subscription_receiver) =
        PubsubClient::slot_subscribe(ws_url)?;

    thread::spawn(move || loop {
        match slot_subscription_receiver.recv() {
            Ok(result) => {
                println!("slot subscription result: {:?}", result);
            }
            Err(e) => {
                println!("slot subscription error: {:?}", e);
                break;
            }
        }
    });

    println!("-------------------- slot updates subscription --------------------");
    let (slot_updates_subscription_sender, slot_updates_subscription_receiver) =
        unbounded::<SlotInfo>();
    let handler = move |message| match slot_updates_subscription_sender.send(message) {
        Ok(_) => (),
        Err(e) => {
            println!("handler receive error: {:?}", e);
        }
    };

    let start_slot = SlotInfo {
        slot: 95546,
        parent: 95545,
        root: 95514,
    };
    let mut slot_updates_subscription_client =
        PubsubClient::slot_updates_subscribe(ws_url, move |_| handler(start_slot))?;

    thread::spawn(move || loop {
        match slot_updates_subscription_receiver.recv() {
            Ok(result) => {
                println!("slot updates subscription result: {:?}", result);
            }
            Err(e) => {
                println!("slot updates subscription error: {:?}", e);
                break;
            }
        }
    });

    /*
        // error with block_subscription and vote_subscription:
        // Error: unexpected message format: {"error": Object({"code":
        // Number(-32601), "message": String("Method not found")}), "id":
        // Number(1), "jsonrpc": String("2.0")}
        println!("-------------------- block subscription --------------------");
        let (mut block_subscription_client, block_subscription_receiver) = PubsubClient::block_subscribe(
            ws_url,
            RpcBlockSubscribeFilter::All,
            Some(RpcBlockSubscribeConfig {
                commitment: Some(CommitmentConfig {
                    commitment: CommitmentLevel::Confirmed,
                }),
                encoding: Some(UiTransactionEncoding::Json),
                transaction_details: Some(TransactionDetails::Signatures),
                show_rewards: None,
                max_supported_transaction_version: None,
            }),
        )?;

        thread::spawn(move || {
            loop {
                match block_subscription_receiver.recv() {
                    Ok(result) => {
                        println!("block subscription result: {:?}", result);
                    }
                    Err(e) => {
                        println!("block subscription error: {:?}", e);
                        break;
                    }
                }
            }
        });

        block_subscription_client.send_unsubscribe();
        block_subscription_client.shutdown();

        println!("-------------------- vote subscription --------------------");
        let (mut vote_subscription_client, vote_subscription_receiver) = PubsubClient::vote_subscribe(ws_url)?;

        thread::spawn(move || {
            loop {
                match vote_subscription_receiver.recv() {
                    Ok(result) => {
                        println!("vote subscription result: {:?}", result);
                    }
                    Err(e) => {
                        println!("vote subscription error: {:?}", e);
                        break;
                    }
                }
            }
        });

        vote_subscription_client.send_unsubscribe();
        vote_subscription_client.shutdown();
    */

    loop {
        thread::yield_now();
    }

    println!("-------------------- clients unsubscribe and shutdown --------------------");

    account_subscription_client.send_unsubscribe();
    account_subscription_client.shutdown();

    account_subscription_client_for_alice.send_unsubscribe();
    account_subscription_client_for_alice.shutdown();

    logs_subscription_client.send_unsubscribe();
    logs_subscription_client.shutdown();

    program_subscription_client.send_unsubscribe();
    program_subscription_client.shutdown();

    root_subscription_client.send_unsubscribe();
    root_subscription_client.shutdown();

    sig_subscription_client.send_unsubscribe();
    sig_subscription_client.shutdown();

    slot_subscription_client.send_unsubscribe();
    slot_subscription_client.shutdown();

    slot_updates_subscription_client.shutdown();

    Ok(())
}
