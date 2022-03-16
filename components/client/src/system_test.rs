use anyhow::Result;
use common::system_test::{Allocate, CreateAccount, TransferLamports, TransferLamportsToMany};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, signature::Signer, signer::keypair::Keypair, system_instruction,
    system_program, transaction::Transaction,
};

pub fn create_account_via_program(
    client: &RpcClient,
    program_id: &Pubkey,
    payer: &Keypair,
    new_account: &Keypair,
) -> Result<()> {
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
        &[payer, new_account],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("create_account_via_program tx signature: {:#?}", sig);

    let account = client.get_account(&new_account.pubkey())?;
    println!("new_account_for_program_test created: {:#?}", account);

    Ok(())
}

pub fn create_account_via_rpc(
    client: &RpcClient,
    payer: &Keypair,
    new_account: &Keypair,
) -> Result<()> {
    println!("--------------------------------------- create_account_via_rpc ---------------------------------------");

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
        &[payer, new_account],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("create_account_via_rpc tx signature: {:#?}", sig);

    let account = client.get_account(&new_account.pubkey())?;
    println!("new_account_for_rpc_test created: {:#?}", account);

    Ok(())
}

pub fn allocate_via_program(
    client: &RpcClient,
    program_id: &Pubkey,
    payer: &Keypair,
    new_account: &Keypair,
    space: u64,
) -> Result<()> {
    let instr =
        Allocate::build_instruction(program_id, &payer.pubkey(), &new_account.pubkey(), space)?;

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&payer.pubkey()),
        &[payer, new_account],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("allocate_via_program tx signature: {:#?}", sig);

    let account = client.get_account(&new_account.pubkey())?;
    println!("account after allocate_via_program: {:#?}", account);

    Ok(())
}

pub fn allocate_via_rpc(
    client: &RpcClient,
    payer: &Keypair,
    new_account: &Keypair,
    space: u64,
) -> Result<()> {
    println!("--------------------------------------- allocate_via_rpc ---------------------------------------");

    let rent = client.get_minimum_balance_for_rent_exemption(space.try_into()?)?;

    let instr1 = system_instruction::transfer(&payer.pubkey(), &new_account.pubkey(), rent);

    let instr2 = system_instruction::allocate(&new_account.pubkey(), space);

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[instr1, instr2],
        Some(&payer.pubkey()),
        &[payer, new_account],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("allocate_via_rpc tx signature: {:#?}", sig);

    let account = client.get_account(&new_account.pubkey())?;
    println!("account after allocate_via_rpc: {:#?}", account);

    Ok(())
}

pub fn transfer_via_program(
    client: &RpcClient,
    program_id: &Pubkey,
    from: &Keypair,
    to: &Pubkey,
    amount: u64,
) -> Result<()> {
    let instr = TransferLamports::build_instruction(program_id, &from.pubkey(), to, amount)?;

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&[instr], Some(&from.pubkey()), &[from], blockhash);

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("transfer_via_program tx signature: {:#?}", sig);

    let account = client.get_account(to)?;
    println!("account after transfer_via_program: {:#?}", account);

    Ok(())
}

pub fn transfer_via_rpc(
    client: &RpcClient,
    from: &Keypair,
    to: &Pubkey,
    amount: u64,
) -> Result<()> {
    println!("--------------------------------------- transfer_via_rpc ---------------------------------------");

    let instr = system_instruction::transfer(&from.pubkey(), to, amount);

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&[instr], Some(&from.pubkey()), &[from], blockhash);

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("transfer_via_rpc tx signature: {:#?}", sig);

    let account = client.get_account(to)?;
    println!("account after transfer_via_rpc: {:#?}", account);

    Ok(())
}

pub fn transfer_many_via_program(
    client: &RpcClient,
    program_id: &Pubkey,
    from: &Keypair,
    to_and_amount: &[(Pubkey, u64)],
) -> Result<()> {
    println!("to_and_amount: {:#?}", to_and_amount);
    let instr = TransferLamportsToMany::build_instruction(program_id, &from.pubkey(), to_and_amount)?;

    println!("instr: {:#?}", instr);
    
    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&[instr], Some(&from.pubkey()), &[from], blockhash);

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("transfer_many_via_program tx signature: {:#?}", sig);

    for (to_pubkey, _) in to_and_amount {
        let account = client.get_account(to_pubkey)?;
        println!("account after transfer_many_via_program: {:#?}", account);
    }

    Ok(())
}

pub fn transfer_many_via_rpc(
    client: &RpcClient,
    from: &Keypair,
    to_and_amount: &[(Pubkey, u64)],
) -> Result<()> {
    println!("--------------------------------------- transfer_many_via_rpc ---------------------------------------");

    let from_pubkey = from.pubkey();
    let instr = to_and_amount
        .iter()
        .map(|(to_pubkey, amount)| system_instruction::transfer(&from_pubkey, to_pubkey, *amount))
        .collect::<Vec<_>>();

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(&instr, Some(&from.pubkey()), &[from], blockhash);

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("transfer_many_via_rpc tx signature: {:#?}", sig);

    for (to_pubkey, _) in to_and_amount {
        let account = client.get_account(to_pubkey)?;
        println!("account after transfer_many_via_program: {:#?}", account);
    }

    Ok(())
}
