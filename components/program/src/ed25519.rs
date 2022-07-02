use common::DemoEd25519Instruction;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    ed25519_program,
    entrypoint::ProgramResult,
    keccak, msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

pub fn demo_ed25519(
    program_id: &Pubkey,
    instruction: DemoEd25519Instruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo ed25519");

    let account_info_iter = &mut accounts.iter();
    let instructions_sysvar_account = next_account_info(account_info_iter)?;
    assert!(sysvar::instructions::check_id(
        instructions_sysvar_account.key
    ));

    Ok(())
}
