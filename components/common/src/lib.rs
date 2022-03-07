use borsh::{BorshDeserialize, BorshSerialize};
use system_test::SystemTestInstruction;
use sysvar_test::SysvarTestInstruction;

pub mod system_test;
pub mod sysvar_test;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum ProgramInstruction {
    SystemTest(SystemTestInstruction),
    SysvarTest(SysvarTestInstruction),
}
