use common::{
    DemoSystemProgramCreateAccountInstruction,
    DemoSystemProgramTransferAllocAssignInstruction,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey,
    account_info::next_account_info,
    msg,
    system_program,
    system_instruction,
    sysvar::rent::Rent,
    program::invoke,
    sysvar::Sysvar,
};

pub fn demo_system_program_create_account(
    program_id: &Pubkey,
    _instruction: DemoSystemProgramCreateAccountInstruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo secp256k1 verify basic");

    let account_info_iter = &mut accounts.iter();

    let system_program_account = next_account_info(account_info_iter)?;
    assert!(system_program::check_id(
        system_program_account.key
    ));

    let payer = next_account_info(account_info_iter)?;
    assert!(payer.is_signer);
    assert!(payer.is_writable);

    let new_account = next_account_info(account_info_iter)?;
    assert!(new_account.is_signer);
    assert!(new_account.is_writable);

    let space = 1;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    let instr = system_instruction::create_account(
        payer.key,
        new_account.key,
        lamports,
        space as u64,
        program_id,
    );

    invoke(
        &instr,
        &[payer.clone(), new_account.clone()],
    )?;

    msg!("new account: {}", new_account.key);
    msg!("lamports: {}", lamports);

    Ok(())
}

pub fn demo_system_program_transfer_alloc_assign(
    program_id: &Pubkey,
    _instruction: DemoSystemProgramTransferAllocAssignInstruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo secp256k1 verify basic");

    let account_info_iter = &mut accounts.iter();

    let system_program_account = next_account_info(account_info_iter)?;
    assert!(system_program::check_id(
        system_program_account.key
    ));

    let payer = next_account_info(account_info_iter)?;
    assert!(payer.is_signer);
    assert!(payer.is_writable);

    let new_account = next_account_info(account_info_iter)?;
    assert!(new_account.is_signer);
    assert!(new_account.is_writable);

    let space = 1;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    let transfer_instr = system_instruction::transfer(
        payer.key,
        new_account.key,
        lamports,
    );

    let alloc_instr = system_instruction::allocate(
        new_account.key,
        space as u64,
    );

    let assign_instr = system_instruction::assign(
        new_account.key,
        program_id,
    );

    invoke(
        &transfer_instr,
        &[payer.clone(), new_account.clone()],
    )?;

    invoke(
        &alloc_instr,
        &[new_account.clone()],
    )?;

    invoke(
        &assign_instr,
        &[new_account.clone()],
    )?;

    msg!("new account: {}", new_account.key);
    msg!("lamports: {}", lamports);

    Ok(())
}
