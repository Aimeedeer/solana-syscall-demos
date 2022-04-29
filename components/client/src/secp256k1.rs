use anyhow::Result;
use common::DemoSecp256k1Instruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub fn demo_secp256k1(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let secret_key = libsecp256k1::SecretKey::random(&mut rand::thread_rng());
    let msg = b"hello world";
    let verify_secp256k1_instr =
        solana_sdk::secp256k1_instruction::new_secp256k1_instruction(&secret_key, msg);

    let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);
    let public_key = solana_sdk::secp256k1_instruction::construct_eth_pubkey(&public_key);
    let program_instr = DemoSecp256k1Instruction::build_instruction(
        &program_keypair.pubkey(),
        msg.to_vec(),
        public_key,
    )?;

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[verify_secp256k1_instr, program_instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}
