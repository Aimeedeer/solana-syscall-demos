use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use common::{
    DemoSystemProgramCreateAccountInstruction,
    DemoSystemProgramTransferAllocAssignInstruction,
};

pub fn demo_system_program_create_account_cpi(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let new_account = Keypair::new();

    let instr = DemoSystemProgramCreateAccountInstruction {
        payer: config.keypair.pubkey(),
        new_account: new_account.pubkey(),
    }.build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair, &new_account],
        blockhash,
    );

    println!("new account: {}", new_account.pubkey());

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}

pub fn demo_system_program_transfer_alloc_assign_cpi(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let new_account = Keypair::new();

    let instr = DemoSystemProgramTransferAllocAssignInstruction {
        payer: config.keypair.pubkey(),
        new_account: new_account.pubkey(),
    }.build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair, &new_account],
        blockhash,
    );

    println!("new account: {}", new_account.pubkey());

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}
