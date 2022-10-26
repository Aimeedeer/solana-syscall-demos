use common::{
    DemoSystemProgramCreateAccountInstruction, DemoSystemProgramTransferAllocAssignInstruction,
};
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, entrypoint::ProgramResult, msg,
    program::invoke_signed, pubkey::Pubkey, system_instruction, system_program, sysvar::rent::Rent,
    sysvar::Sysvar,
};

pub fn demo_system_program_create_account(
    program_id: &Pubkey,
    instruction: DemoSystemProgramCreateAccountInstruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo secp256k1 verify basic");

    let account_info_iter = &mut accounts.iter();

    let system_program_account = next_account_info(account_info_iter)?;
    assert!(system_program::check_id(system_program_account.key));

    let payer = next_account_info(account_info_iter)?;
    assert!(payer.is_signer);
    assert!(payer.is_writable);

    let new_account_pda = next_account_info(account_info_iter)?;
    assert!(!new_account_pda.is_signer);
    assert!(new_account_pda.is_writable);

    let new_account_seed = &instruction.new_account_seed;
    let new_account_bump_seed = instruction.new_account_bump_seed;

    let space = 1;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    let instr = system_instruction::create_account(
        payer.key,
        new_account_pda.key,
        lamports,
        space as u64,
        program_id,
    );

    invoke_signed(
        &instr,
        &[payer.clone(), new_account_pda.clone()],
        &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
    )?;

    msg!("new account: {}", new_account_pda.key);
    msg!("lamports: {}", lamports);

    Ok(())
}

pub fn demo_system_program_transfer_alloc_assign(
    program_id: &Pubkey,
    instruction: DemoSystemProgramTransferAllocAssignInstruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo secp256k1 verify basic");

    let account_info_iter = &mut accounts.iter();

    let system_program_account = next_account_info(account_info_iter)?;
    assert!(system_program::check_id(system_program_account.key));

    let payer = next_account_info(account_info_iter)?;
    assert!(payer.is_signer);
    assert!(payer.is_writable);

    let new_account_pda = next_account_info(account_info_iter)?;
    assert!(!new_account_pda.is_signer);
    assert!(new_account_pda.is_writable);

    let new_account_seed = &instruction.new_account_seed;
    let new_account_bump_seed = instruction.new_account_bump_seed;

    let space = 1;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    let transfer_instr = system_instruction::transfer(payer.key, new_account_pda.key, lamports);

    let alloc_instr = system_instruction::allocate(new_account_pda.key, space as u64);

    let assign_instr = system_instruction::assign(new_account_pda.key, program_id);

    invoke_signed(
        &transfer_instr,
        &[payer.clone(), new_account_pda.clone()],
        &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
    )?;

    invoke_signed(
        &alloc_instr,
        &[new_account_pda.clone()],
        &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
    )?;

    invoke_signed(
        &assign_instr,
        &[new_account_pda.clone()],
        &[&[payer.key.as_ref(), new_account_seed, &[new_account_bump_seed]]],
    )?;

    msg!("new account: {}", new_account_pda.key);
    msg!("lamports: {}", lamports);

    Ok(())
}
