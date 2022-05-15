use common::DemoInvokeInstruction;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult,
    msg,
};

pub fn demo_invoke(
    _instruction: DemoInvokeInstruction,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo invoke");

    Ok(())
}
