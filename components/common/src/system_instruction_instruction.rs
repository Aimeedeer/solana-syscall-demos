use borsh::{BorshDeserialize, BorshSerialize};

/// # Accounts
///
/// - 0: payer - writable, signer
/// - 1: system_program - executable
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SystemInstructionInstruction {
    pub rent_lamports: u64,
}
