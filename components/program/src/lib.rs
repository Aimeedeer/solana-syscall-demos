use borsh::de::BorshDeserialize;
use common::CustomInstruction;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

mod ed25519;
mod invoke;
mod secp256k1;
mod sysvars;

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = {
        let mut instruction_data = instruction_data;
        CustomInstruction::deserialize(&mut instruction_data)?
    };

    match instruction {
        CustomInstruction::PrintSysvars(instr) => {
            sysvars::print_sysvars(instr, accounts, instruction_data)?;
        }
        CustomInstruction::DemoSecp256k1VerifyBasic(instr) => {
            secp256k1::demo_secp256k1_verify_basic(instr, accounts)?;
        }
        CustomInstruction::DemoSecp256k1CustomMany(instr) => {
            secp256k1::demo_secp256k1_custom_many(instr, accounts)?;
        }
        CustomInstruction::DemoSecp256k1Recover(instr) => {
            secp256k1::demo_secp256k1_recover(instr, accounts)?;
        }
        CustomInstruction::DemoEd25519(instr) => {
            ed25519::demo_ed25519(program_id, instr, accounts)?;
        }
        CustomInstruction::DemoInvoke(instr) => {
            invoke::demo_invoke(program_id, instr, accounts)?;
        }
    }

    Ok(())
}
