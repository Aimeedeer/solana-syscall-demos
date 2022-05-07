use common::{DemoSecp256k1RecoverInstruction, DemoSecp256k1VerifyBasicInstruction};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    keccak, msg,
    program_error::ProgramError,
    secp256k1_recover::secp256k1_recover,
};

/// Definitions copied from solana-sdk
mod defs {
    use serde_derive::Deserialize;

    pub const HASHED_PUBKEY_SERIALIZED_SIZE: usize = 20;
    //const SIGNATURE_SERIALIZED_SIZE: usize = 64;
    pub const SIGNATURE_OFFSETS_SERIALIZED_SIZE: usize = 11;

    #[allow(unused)]
    #[derive(Deserialize)]
    pub struct SecpSignatureOffsets {
        pub signature_offset: u16, // offset to [signature,recovery_id] of 64+1 bytes
        pub signature_instruction_index: u8,
        pub eth_address_offset: u16, // offset to eth_address of 20 bytes
        pub eth_address_instruction_index: u8,
        pub message_data_offset: u16, // offset to start of message data
        pub message_data_size: u16,   // size of message data
        pub message_instruction_index: u8,
    }
}

pub fn demo_secp256k1_verify_basic(
    instruction: DemoSecp256k1VerifyBasicInstruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo secp256k1");

    use solana_program::secp256k1_program;
    use solana_program::sysvar;

    let account_info_iter = &mut accounts.iter();

    let instructions_sysvar_account = next_account_info(account_info_iter)?;
    assert!(sysvar::instructions::check_id(
        instructions_sysvar_account.key
    ));

    // `new_secp256k1_instruction` generates an instruction that must be at index 0.
    let secp256k1_instr =
        sysvar::instructions::load_instruction_at_checked(0, instructions_sysvar_account)?;

    // Verify it is a secp256k1 instruction.
    assert!(secp256k1_program::check_id(&secp256k1_instr.program_id));

    // There must be at least one byte.
    assert!(secp256k1_instr.data.len() > 1);

    let num_signatures = secp256k1_instr.data[0];
    // `new_secp256k1_instruction` generates an instruction that contains one signature.
    assert_eq!(1, num_signatures);

    let offsets_slice = &secp256k1_instr.data[1..defs::SIGNATURE_OFFSETS_SERIALIZED_SIZE + 1];

    let offsets: defs::SecpSignatureOffsets =
        bincode::deserialize(offsets_slice).expect("deserialize");

    // `new_secp256k1_instruction` generates an instruction that only uses instruction index 0.
    assert_eq!(0, offsets.signature_instruction_index);
    assert_eq!(0, offsets.eth_address_instruction_index);
    assert_eq!(0, offsets.message_instruction_index);

    // Verify the public key we expect signed the message we expect.
    // These are the checks that are ultimately required for a program
    // to verify a signature.
    //
    // Checking these verifies that `verified_pubkey` signed `verified_message`.

    let verified_pubkey = &secp256k1_instr.data[usize::from(offsets.eth_address_offset)
        ..usize::from(offsets.eth_address_offset)
            .saturating_add(defs::HASHED_PUBKEY_SERIALIZED_SIZE)];
    let verified_message = &secp256k1_instr.data[usize::from(offsets.message_data_offset)
        ..usize::from(offsets.message_data_offset)
            .saturating_add(usize::from(offsets.message_data_size))];

    assert_eq!(&instruction.signer_pubkey[..], verified_pubkey);
    assert_eq!(&instruction.message[..], verified_message);

    Ok(())
}

pub fn demo_secp256k1_recover(
    instruction: DemoSecp256k1RecoverInstruction,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    let message_hash = {
        let mut hasher = keccak::Hasher::default();
        hasher.hash(&instruction.message);
        hasher.result()
    };

    let recovered_pubkey = secp256k1_recover(
        &message_hash.0,
        instruction.recovery_id,
        &instruction.signature,
    )
    .map_err(|_| ProgramError::InvalidArgument)?;

    msg!(
        "recovered signer pubkey: {}",
        hex::encode(recovered_pubkey.0)
    );
    msg!(
        "expected signer pubkey: {}",
        hex::encode(instruction.expected_signer_pubkey)
    );

    assert_eq!(recovered_pubkey.0, instruction.expected_signer_pubkey);

    Ok(())
}
