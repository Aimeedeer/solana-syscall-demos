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
    DemoSecp256k1Basic(DemoSecp256k1BasicInstruction),
    DemoSecp256k1Recover(DemoSecp256k1RecoverInstruction),
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
    pub section: PrintSysvarsSection,
}

/// This is just used to break up execution to fit in the CPU budget.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum PrintSysvarsSection {
    One,
}

impl PrintSysvarsInstruction {
    pub fn build_instruction(self, program_id: &Pubkey) -> Result<Instruction> {
        let instr = CustomInstruction::PrintSysvars(self);
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

/// # Accounts
///
/// - 0: instructions sysvar
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DemoSecp256k1BasicInstruction {
    pub message: Vec<u8>,
    pub signer_pubkey: [u8; 20],
}

impl DemoSecp256k1BasicInstruction {
    pub fn build_instruction(self, program_id: &Pubkey) -> Result<Instruction> {
        let instr = CustomInstruction::DemoSecp256k1Basic(self);
        let accounts = vec![AccountMeta::new_readonly(sysvar::instructions::ID, false)];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}

/// # Accounts
///
/// None
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DemoSecp256k1RecoverInstruction {
    pub message: Vec<u8>,
    pub signature: [u8; 64],
    pub recovery_id: u8,
    pub expected_signer_pubkey: [u8; 64],
}

impl DemoSecp256k1RecoverInstruction {
    pub fn build_instruction(self, program_id: &Pubkey) -> Result<Instruction> {
        let instr = CustomInstruction::DemoSecp256k1Recover(self);
        let accounts = vec![];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}
