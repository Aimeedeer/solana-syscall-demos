use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

/// # Accounts
///
/// - 0: payer - writable, signer
/// - 1: system_program - executable
/// - 2: clock - executable
/// - 3: epoch_schedule - executable
/// - 4: rent - executable
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CustomInstruction {
    test_amount: u64,
}

impl CustomInstruction {
    pub fn build_instruction(payer: &Pubkey, program_id: &Pubkey) -> Result<Instruction> {
        let instr = CustomInstruction { test_amount: 1_000 };

        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
            AccountMeta::new_readonly(sysvar::epoch_schedule::ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}
