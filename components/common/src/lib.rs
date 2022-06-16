use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CustomInstruction {
    PrintSysvars(PrintSysvarsInstruction),
    DemoSecp256k1VerifyBasic(DemoSecp256k1VerifyBasicInstruction),
    DemoSecp256k1Recover(DemoSecp256k1RecoverInstruction),
    DemoInvoke(DemoInvokeInstruction),
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
#[derive(BorshSerialize, BorshDeserialize, Debug, Copy, Clone)]
pub enum PrintSysvarsSection {
    One,
    Two,
    Three,
}

impl PrintSysvarsInstruction {
    pub fn build_instruction(self, program_id: &Pubkey) -> Instruction {
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

        Instruction::new_with_borsh(*program_id, &instr, accounts)
    }
}

/// # Accounts
///
/// - 0: instructions sysvar
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DemoSecp256k1VerifyBasicInstruction {}

impl DemoSecp256k1VerifyBasicInstruction {
    pub fn build_instruction(self, program_id: &Pubkey) -> Instruction {
        let instr = CustomInstruction::DemoSecp256k1VerifyBasic(self);
        let accounts = vec![AccountMeta::new_readonly(sysvar::instructions::ID, false)];

        Instruction::new_with_borsh(*program_id, &instr, accounts)
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
}

impl DemoSecp256k1RecoverInstruction {
    pub fn build_instruction(self, program_id: &Pubkey) -> Instruction {
        let instr = CustomInstruction::DemoSecp256k1Recover(self);
        let accounts = vec![];

        Instruction::new_with_borsh(*program_id, &instr, accounts)
    }
}

/// # Accounts
///
/// - 0: this program id - executable
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct DemoInvokeInstruction {
    pub mode: DemoInvokeMode,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum DemoInvokeMode {
    Caller,
    Callee,
}

impl DemoInvokeInstruction {
    pub fn build_instruction(self, program_id: &Pubkey) -> Instruction {
        let instr = CustomInstruction::DemoInvoke(self);
        let accounts = vec![AccountMeta::new_readonly(program_id.clone(), false)];

        Instruction::new_with_borsh(*program_id, &instr, accounts)
    }
}
