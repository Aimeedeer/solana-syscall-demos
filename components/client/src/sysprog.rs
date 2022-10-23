use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
};
use common::{
    DemoSystemProgramCreateAccountInstruction,
    DemoSystemProgramTransferAllocAssignInstruction,
};
use rand::Rng;

pub fn demo_system_program_create_account_cpi(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let new_account_seed: [u8; 16] = rand::thread_rng().gen();
    let (new_account_pda, new_account_bumpkey) = Pubkey::find_program_address(
        &[
            config.keypair.pubkey().as_ref(),
            &new_account_seed,
        ],
        &program_keypair.pubkey(),
    );

    let instr = DemoSystemProgramCreateAccountInstruction {
        payer: config.keypair.pubkey(),
        new_account_pda,
        new_account_seed,
        new_account_bumpkey,
    }.build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    println!("new account: {}", new_account_pda);

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}

pub fn demo_system_program_transfer_alloc_assign_cpi(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let new_account_seed: [u8; 16] = rand::thread_rng().gen();
    let (new_account_pda, new_account_bumpkey) = Pubkey::find_program_address(
        &[
            config.keypair.pubkey().as_ref(),
            &new_account_seed,
        ],
        &program_keypair.pubkey(),
    );

    let instr = DemoSystemProgramTransferAllocAssignInstruction {
        payer: config.keypair.pubkey(),
        new_account_pda,
        new_account_seed,
        new_account_bumpkey,
    }.build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    println!("new account: {}", new_account_pda);

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}
