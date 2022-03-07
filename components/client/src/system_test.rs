use anyhow::Result;
use common::system_test::CreateAccount;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, signature::Signer, signer::keypair::Keypair, system_instruction,
    system_program, transaction::Transaction,
};

pub fn create_account_via_program(
    client: &RpcClient,
    program_id: &Pubkey,
    payer: &Keypair,
) -> Result<()> {
    let new_account = Keypair::new();
    let space = 0;

    let instr = CreateAccount::build_instruction(
        program_id,
        &payer.pubkey(),
        &new_account.pubkey(),
        space,
    )?;

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&payer.pubkey()),
        &[payer, &new_account],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("create_account_via_program tx signature: {:#?}", sig);

    Ok(())
}

pub fn create_account_via_rpc(client: &RpcClient, payer: &Keypair) -> Result<()> {
    println!("--------------------------------------- create_account_via_rpc ---------------------------------------");

    let new_account = Keypair::new();
    let space = 0;

    let rent = client.get_minimum_balance_for_rent_exemption(space.try_into()?)?;
    let instr = system_instruction::create_account(
        &payer.pubkey(),
        &new_account.pubkey(),
        rent,
        space,
        &system_program::ID,
    );

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&payer.pubkey()),
        &[payer, &new_account],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("create_account_via_rpc tx signature: {:#?}", sig);

    Ok(())
}
