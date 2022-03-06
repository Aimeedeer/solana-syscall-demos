use borsh::{BorshDeserialize, BorshSerialize};
use system_instruction_instruction::SystemInstructionInstruction;
use sysvar_instruction::SysvarInstruction;

pub mod system_instruction_instruction;
pub mod sysvar_instruction;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum ProgramInstruction {
    SystemInstruction(SystemInstructionInstruction),
    Sysvar(SysvarInstruction),
}
