use common::DemoSystemProgramCreateAccountInstruction;
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
    _program_id: &Pubkey,
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
        &system_program::ID,
    );

    invoke(
        &instr,
        &[payer.clone(), new_account.clone()],
    )?;

    msg!("new account: {}", new_account.key);
    msg!("lamports: {}", lamports);

    Ok(())
}
