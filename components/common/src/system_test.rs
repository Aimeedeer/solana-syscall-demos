use crate::ProgramInstruction;
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum SystemTestInstruction {
    CreateAccount(CreateAccount),
    Allocate(Allocate),
    TransferLamports(TransferLamports),
    TransferLamportsToMany(TransferLamportsToMany),
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

/// # Accounts
///
/// - 0: payer - writable, signer
/// - 1: new_account - writable, signer
/// - 2: system_program - executable
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Allocate {
    pub space: u64,
}

/// # Accounts
///
/// - 0: from - writable, signer
/// - 1: to - writable
/// - 2: system_program - executable
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TransferLamports {
    pub amount: u64,
}

/// # Accounts
///
/// - 0: from - writable, signer
/// - 1: system_program - executable
/// - *: to - writable
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TransferLamportsToMany {
    pub amount_list: Vec<u64>,
}

impl CreateAccount {
    pub fn build_instruction(
        program_id: &Pubkey,
        payer: &Pubkey,
        new_account: &Pubkey,
        space: u64,
    ) -> Result<Instruction> {
        let instr = CreateAccount { space };
        let instr = ProgramInstruction::SystemTest(SystemTestInstruction::CreateAccount(instr));

        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*new_account, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}

impl Allocate {
    pub fn build_instruction(
        program_id: &Pubkey,
        payer: &Pubkey,
        new_account: &Pubkey,
        space: u64,
    ) -> Result<Instruction> {
        let instr = Allocate { space };
        let instr = ProgramInstruction::SystemTest(SystemTestInstruction::Allocate(instr));

        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*new_account, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}

impl TransferLamports {
    pub fn build_instruction(
        program_id: &Pubkey,
        from: &Pubkey,
        to: &Pubkey,
        amount: u64,
    ) -> Result<Instruction> {
        let instr = TransferLamports { amount };
        let instr = ProgramInstruction::SystemTest(SystemTestInstruction::TransferLamports(instr));

        let accounts = vec![
            AccountMeta::new(*from, true),
            AccountMeta::new(*to, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}

impl TransferLamportsToMany {
    pub fn build_instruction(
        program_id: &Pubkey,
        from: &Pubkey,
        to_and_amount: &[(Pubkey, u64)],
    ) -> Result<Instruction> {
        let mut amount_list = vec![];
        let mut accounts = vec![
            AccountMeta::new(*from, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        for (to_pubkey, amount) in to_and_amount {
            accounts.push(AccountMeta::new(*to_pubkey, false));
            amount_list.push(*amount);
        }

        let instr = TransferLamportsToMany { amount_list };
        let instr = ProgramInstruction::SystemTest(SystemTestInstruction::TransferLamportsToMany(instr));
        
        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}
