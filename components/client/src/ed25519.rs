use anyhow::Result;
use common::DemoEd25519Instruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    ed25519_instruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

use ed25519_dalek::{Keypair as Ed25519Keypair, Signer as Ed25519Signer, KEYPAIR_LENGTH};

const ED25519_KEYPAIR: [u8; KEYPAIR_LENGTH] = [
    200, 44, 197, 236, 56, 17, 29, 59, 168, 204, 169, 156, 9, 18, 216, 0, 165, 242, 19, 167, 30,
    32, 68, 205, 83, 19, 195, 87, 198, 224, 114, 103, 211, 210, 72, 176, 173, 140, 129, 224, 36,
    99, 29, 4, 141, 117, 74, 94, 173, 213, 199, 210, 26, 108, 206, 227, 55, 76, 126, 162, 14, 112,
    100, 112,
];

pub fn demo_ed25519_instruction(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let message: &[u8] = b"This is a demo message.";

    let keypair = Ed25519Keypair::from_bytes(&ED25519_KEYPAIR)?;
    let _signature = keypair.sign(message);

    let ed25519_instr = ed25519_instruction::new_ed25519_instruction(&keypair, message);
    let program_instr = DemoEd25519Instruction.build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[ed25519_instr, program_instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}
