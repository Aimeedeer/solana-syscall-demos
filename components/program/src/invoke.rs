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

    let return_data = program::get_return_data();

    if let Some((return_data_pubkey, return_data)) = return_data {
        msg!(
            "return data: ({}, {:?})",
            return_data_pubkey,
            std::str::from_utf8(&return_data)
        );
        assert_eq!(&return_data_pubkey, program_id);
        assert_eq!(return_data, b"hello world");
    } else {
        panic!("expected return data");
    }

    Ok(())
}

fn do_callee_mode() -> ProgramResult {
    msg!("invoke callee");

    program::set_return_data(b"hello world");

    Ok(())
}
