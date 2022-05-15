use common::{DemoInvokeInstruction, DemoInvokeMode};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program, pubkey::Pubkey,
};

pub fn demo_invoke(
    program_id: &Pubkey,
    instruction: DemoInvokeInstruction,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    match instruction.mode {
        DemoInvokeMode::Caller => {
            do_caller_mode(program_id)?;
        }
        DemoInvokeMode::Callee => {
            do_callee_mode()?;
        }
    }

    Ok(())
}

fn do_caller_mode(program_id: &Pubkey) -> ProgramResult {
    msg!("invoke caller");

    let instr = DemoInvokeInstruction {
        mode: DemoInvokeMode::Callee,
    }
    .build_instruction(program_id);

    program::invoke(&instr, &[])?;

    Ok(())
}

fn do_callee_mode() -> ProgramResult {
    msg!("invoke callee");

    Ok(())
}
