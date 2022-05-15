use anyhow::Result;
use common::{DemoInvokeInstruction, DemoInvokeMode};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub fn demo_invoke(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let instr = DemoInvokeInstruction {
        mode: DemoInvokeMode::Caller,
    }
    .build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}
