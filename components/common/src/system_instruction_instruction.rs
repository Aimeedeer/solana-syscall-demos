use crate::ProgramInstruction;
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum SystemInstructionInstruction {
    CreateAccount(CreateAccount),
}

/// # Accounts
///
/// - 0: payer - writable, signer
/// - 1: new_account - writable, signer
/// - 2: system_program - executable
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CreateAccount {
    pub space: u64,
}

impl CreateAccount {
    pub fn build_instruction(
        program_id: &Pubkey,
        payer: &Pubkey,
        new_account: &Pubkey,
        space: u64,
    ) -> Result<Instruction> {
        let instr = CreateAccount { space };
        let instr = ProgramInstruction::SystemInstruction(
            SystemInstructionInstruction::CreateAccount(instr),
        );

        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*new_account, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}
