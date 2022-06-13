use common::{DemoSecp256k1RecoverInstruction, DemoSecp256k1VerifyBasicInstruction};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    keccak, msg,
    program_error::ProgramError,
    secp256k1_recover::secp256k1_recover,
};

/// Definitions copied from solana-sdk
mod secp256k1_defs {
    use solana_program::program_error::ProgramError;
    use std::iter::Iterator;

    pub const HASHED_PUBKEY_SERIALIZED_SIZE: usize = 20;
    pub const SIGNATURE_SERIALIZED_SIZE: usize = 64;
    pub const SIGNATURE_OFFSETS_SERIALIZED_SIZE: usize = 11;

    pub fn iter_signature_offsets(
        secp256k1_instr_data: &[u8],
    ) -> Result<impl Iterator<Item = SecpSignatureOffsets> + '_, ProgramError> {
        let num_structs = *secp256k1_instr_data
            .get(0)
            .ok_or(ProgramError::InvalidArgument)?;
        let all_structs_size = SIGNATURE_OFFSETS_SERIALIZED_SIZE * num_structs as usize;
        let all_structs_slice = secp256k1_instr_data
            .get(1..all_structs_size + 1)
            .ok_or(ProgramError::InvalidArgument)?;

        fn decode_u16(chunk: &[u8], index: usize) -> u16 {
            u16::from_le_bytes(<[u8; 2]>::try_from(&chunk[index..index + 2]).unwrap())
        }

        Ok(all_structs_slice
            .chunks(SIGNATURE_OFFSETS_SERIALIZED_SIZE)
            .map(|chunk| SecpSignatureOffsets {
                signature_offset: decode_u16(chunk, 0),
                signature_instruction_index: chunk[2],
                eth_address_offset: decode_u16(chunk, 3),
                eth_address_instruction_index: chunk[5],
                message_data_offset: decode_u16(chunk, 6),
                message_data_size: decode_u16(chunk, 8),
                message_instruction_index: chunk[10],
            }))
    }

    pub struct SecpSignatureOffsets {
        pub signature_offset: u16,
        pub signature_instruction_index: u8,
        pub eth_address_offset: u16,
        pub eth_address_instruction_index: u8,
        pub message_data_offset: u16,
        pub message_data_size: u16,
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
    // This is security-critical - what if the transaction uses an imposter secp256k1 program?
    assert!(secp256k1_program::check_id(&secp256k1_instr.program_id));

    // There must be at least one byte.
    assert!(secp256k1_instr.data.len() > 1);

    let num_signatures = secp256k1_instr.data[0];
    // `new_secp256k1_instruction` generates an instruction that contains one signature.
    assert_eq!(1, num_signatures);

    let offsets: secp256k1_defs::SecpSignatureOffsets =
        secp256k1_defs::iter_signature_offsets(&secp256k1_instr.data)?
            .next()
            .expect("offsets");

    // `new_secp256k1_instruction` generates an instruction that only uses instruction index 0.
    assert_eq!(0, offsets.signature_instruction_index);
    assert_eq!(0, offsets.eth_address_instruction_index);
    assert_eq!(0, offsets.message_instruction_index);

    // Reject high-s value signatures to prevent malleability.
    // Solana does not do this itself.
    // This may or may not be necessary depending on use case.
    {
        let signature = &secp256k1_instr.data[offsets.signature_offset as usize
            ..offsets.signature_offset as usize + secp256k1_defs::SIGNATURE_SERIALIZED_SIZE];
        let signature = libsecp256k1::Signature::parse_standard_slice(signature)
            .map_err(|_| ProgramError::InvalidArgument)?;

        if signature.s.is_high() {
            msg!("signature with high-s value");
            return Err(ProgramError::InvalidArgument);
        }
    }

    // Verify the public key we expect signed the message. Most programs will at
    // least need to verify the pubkey that signed the message is the same as
    // some known pubkey. Otherwise we have only verified that some key signed
    // some message.
    // This is security-critical.

    let verified_pubkey = &secp256k1_instr.data[usize::from(offsets.eth_address_offset)
        ..usize::from(offsets.eth_address_offset)
            .saturating_add(secp256k1_defs::HASHED_PUBKEY_SERIALIZED_SIZE)];

    assert_eq!(&instruction.signer_pubkey[..], verified_pubkey);

    Ok(())
}

pub fn demo_secp256k1_recover(
    instruction: DemoSecp256k1RecoverInstruction,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    // The secp256k1 recovery operation accepts a cryptographically-hashed
    // message only. Passing it anything else is insecure and allows signatures
    // to be forged.
    //
    // This means that the code calling secp256k1_recover must perform the hash
    // itself, and not assume that data passed to it has been properly hashed.
    let message_hash = {
        let mut hasher = keccak::Hasher::default();
        hasher.hash(&instruction.message);
        hasher.result()
    };

    // Reject high-s value signatures to prevent malleability.
    // Solana does not do this itself.
    // This may or may not be necessary depending on use case.
    {
        let signature = libsecp256k1::Signature::parse_standard_slice(&instruction.signature)
            .map_err(|_| ProgramError::InvalidArgument)?;

        if signature.s.is_high() {
            msg!("signature with high-s value");
            return Err(ProgramError::InvalidArgument);
        }
    }

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
