use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CustomInstruction {
    PrintSysvars(PrintSysvarsInstruction),
    DemoSecp256k1(DemoSecp256k1Instruction),
}

/// # Accounts
///
/// - 0: system_program - executable
/// - 1: clock - executable
/// - 2: epoch_schedule - executable
/// - 3: instructions - executable
/// - 4: rent - executable
/// - 5: slot_hashes - executable
/// - 6: slot_history - executable
/// - 7: stake_history - executable
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PrintSysvarsInstruction {
    test_amount: u64,
}

impl PrintSysvarsInstruction {
    pub fn build_instruction(program_id: &Pubkey) -> Result<Instruction> {
        let instr = CustomInstruction::PrintSysvars(PrintSysvarsInstruction { test_amount: 1_000 });

        let accounts = vec![
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
            AccountMeta::new_readonly(sysvar::epoch_schedule::ID, false),
            AccountMeta::new_readonly(sysvar::instructions::ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
            AccountMeta::new_readonly(sysvar::slot_history::ID, false),
            AccountMeta::new_readonly(sysvar::stake_history::ID, false),
        ];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DemoSecp256k1Instruction {}

impl DemoSecp256k1Instruction {
    pub fn build_instruction(program_id: &Pubkey) -> Result<Instruction> {
        let instr = CustomInstruction::DemoSecp256k1(DemoSecp256k1Instruction {});

        let accounts = vec![];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}
