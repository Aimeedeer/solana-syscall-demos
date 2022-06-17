use common::{
    DemoSecp256k1CustomManyInstruction, DemoSecp256k1RecoverInstruction,
    DemoSecp256k1VerifyBasicInstruction,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    keccak, msg,
    program_error::ProgramError,
    secp256k1_program,
    secp256k1_recover::secp256k1_recover,
    sysvar,
};

/// Definitions copied from solana-sdk
mod secp256k1_defs {
    use solana_program::program_error::ProgramError;
    use std::iter::Iterator;

    pub const HASHED_PUBKEY_SERIALIZED_SIZE: usize = 20;
    pub const SIGNATURE_SERIALIZED_SIZE: usize = 64;
    pub const SIGNATURE_OFFSETS_SERIALIZED_SIZE: usize = 11;

    /// The structure encoded in the secp2256k1 instruction data.
    pub struct SecpSignatureOffsets {
        pub signature_offset: u16,
        pub signature_instruction_index: u8,
        pub eth_address_offset: u16,
        pub eth_address_instruction_index: u8,
        pub message_data_offset: u16,
        pub message_data_size: u16,
        pub message_instruction_index: u8,
    }

    pub fn iter_signature_offsets(
        secp256k1_instr_data: &[u8],
    ) -> Result<impl Iterator<Item = SecpSignatureOffsets> + '_, ProgramError> {
        // First element is the number of `SecpSignatureOffsets`.
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
}

/// The key we expect to sign secp256k1 messages.
/// The corresponding secret key is in the client source.
const AUTHORIZED_PUBLIC_KEY: [u8; 64] = [
    0x8C, 0xD6, 0x47, 0xF8, 0xA5, 0xBF, 0x59, 0xA0, 0x4F, 0x77, 0xFA, 0xFA, 0x6C, 0xA0, 0xE6, 0x4D,
    0x94, 0x5B, 0x46, 0x55, 0xA6, 0x2B, 0xB0, 0x6F, 0x10, 0x4C, 0x9E, 0x2C, 0x6F, 0x42, 0x0A, 0xBE,
    0x18, 0xDF, 0x0B, 0xF0, 0x87, 0x42, 0xBA, 0x88, 0xB4, 0xCF, 0x87, 0x5A, 0x35, 0x27, 0xBE, 0x0F,
    0x45, 0xAE, 0xFC, 0x66, 0x9C, 0x2C, 0x6B, 0xF3, 0xEF, 0xCA, 0x5C, 0x32, 0x11, 0xF7, 0x2A, 0xC7,
];

/// The Ethereum address hashed from `AUTHORIZED_PUBLIC_KEY` via `construct_eth_pubkey`.
const AUTHORIZED_ETH_ADDRESS: [u8; 20] = [
    0x18, 0x8a, 0x5c, 0xf2, 0x3b, 0x0e, 0xff, 0xe9, 0xa8, 0xe1, 0x42, 0x64, 0x5b, 0x82, 0x2f, 0x3a,
    0x6b, 0x8b, 0x52, 0x35,
];

pub fn demo_secp256k1_verify_basic(
    _instruction: DemoSecp256k1VerifyBasicInstruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo secp256k1 verify basic");

    let account_info_iter = &mut accounts.iter();

    // The instructions sysvar gives access to the instructions in the transaction.
    let instructions_sysvar_account = next_account_info(account_info_iter)?;
    assert!(sysvar::instructions::check_id(
        instructions_sysvar_account.key
    ));

    // Load the secp256k1 instruction.
    // `new_secp256k1_instruction` generates an instruction that must be at index 0.
    let secp256k1_instr =
        sysvar::instructions::load_instruction_at_checked(0, instructions_sysvar_account)?;

    // Verify it is a secp256k1 instruction.
    // This is security-critical - what if the transaction uses an imposter secp256k1 program?
    assert!(secp256k1_program::check_id(&secp256k1_instr.program_id));

    // There must be at least one byte. This is also verified by the runtime,
    // and doesn't strictly need to be checked.
    assert!(secp256k1_instr.data.len() > 1);

    let num_signatures = secp256k1_instr.data[0];
    // `new_secp256k1_instruction` generates an instruction that contains one signature.
    assert_eq!(1, num_signatures);

    // Load the first and only set of signature offsets.
    let offsets: secp256k1_defs::SecpSignatureOffsets =
        secp256k1_defs::iter_signature_offsets(&secp256k1_instr.data)?
            .next()
            .ok_or(ProgramError::InvalidArgument)?;

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

    // There is likely at least one more verification step a real program needs
    // to do here to ensure it trusts the secp256k1 instruction, e.g.:
    //
    // - verify the tx signer is authorized
    // - verify the secp256k1 signer is authorized

    // Here we are checking the secp256k1 pubkey against a known authorized pubkey.
    let eth_address = &secp256k1_instr.data[offsets.eth_address_offset as usize
        ..offsets.eth_address_offset as usize + secp256k1_defs::HASHED_PUBKEY_SERIALIZED_SIZE];

    if eth_address != AUTHORIZED_ETH_ADDRESS {
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}

/// This is just demoing loading and parsing of many signatures,
/// with no specific goal.
pub fn demo_secp256k1_custom_many(
    _instruction: DemoSecp256k1CustomManyInstruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo secp256k1 custom many");

    let account_info_iter = &mut accounts.iter();

    let instructions_sysvar_account = next_account_info(account_info_iter)?;
    assert!(sysvar::instructions::check_id(
        instructions_sysvar_account.key
    ));

    let secp256k1_instr =
        sysvar::instructions::get_instruction_relative(-1, instructions_sysvar_account)?;

    assert!(secp256k1_program::check_id(&secp256k1_instr.program_id));

    let signatures = load_signatures(&secp256k1_instr.data, instructions_sysvar_account)?;
    for (idx, signature_bundle) in signatures.iter().enumerate() {
        let signature = hex::encode(&signature_bundle.signature);
        let eth_address = hex::encode(&signature_bundle.eth_address);
        let message = hex::encode(&signature_bundle.message);
        msg!("sig {}: {:?}", idx, signature);
        msg!("recid: {}: {}", idx, signature_bundle.recovery_id);
        msg!("eth address {}: {}", idx, eth_address);
        msg!("message {}: {}", idx, message);
    }

    Ok(())
}

pub struct SecpSignature {
    pub signature: [u8; secp256k1_defs::SIGNATURE_SERIALIZED_SIZE],
    pub recovery_id: u8,
    pub eth_address: [u8; secp256k1_defs::HASHED_PUBKEY_SERIALIZED_SIZE],
    pub message: Vec<u8>,
}

/// Load all signatures indicated in the secp256k1 instruction.
///
/// This function is quite inefficient for reloading the same instructions
/// repeatedly and making copies and allocations.
pub fn load_signatures(
    secp256k1_instr_data: &[u8],
    instructions_sysvar_account: &AccountInfo,
) -> Result<Vec<SecpSignature>, ProgramError> {
    let mut sigs = vec![];
    for offsets in secp256k1_defs::iter_signature_offsets(secp256k1_instr_data)? {
        let signature_instr = sysvar::instructions::load_instruction_at_checked(
            offsets.signature_instruction_index as usize,
            instructions_sysvar_account,
        )?;
        let eth_address_instr = sysvar::instructions::load_instruction_at_checked(
            offsets.eth_address_instruction_index as usize,
            instructions_sysvar_account,
        )?;
        let message_instr = sysvar::instructions::load_instruction_at_checked(
            offsets.message_instruction_index as usize,
            instructions_sysvar_account,
        )?;

        // These indexes must all be valid because the runtime already verified them.
        let signature = &signature_instr.data[offsets.signature_offset as usize
            ..offsets.signature_offset as usize + secp256k1_defs::SIGNATURE_SERIALIZED_SIZE];
        let recovery_id = signature_instr.data
            [offsets.signature_offset as usize + secp256k1_defs::SIGNATURE_SERIALIZED_SIZE];
        let eth_address = &eth_address_instr.data[offsets.eth_address_offset as usize
            ..offsets.eth_address_offset as usize + secp256k1_defs::HASHED_PUBKEY_SERIALIZED_SIZE];
        let message = &message_instr.data[offsets.message_data_offset as usize
            ..offsets.message_data_offset as usize + offsets.message_data_size as usize];

        let signature =
            <[u8; secp256k1_defs::SIGNATURE_SERIALIZED_SIZE]>::try_from(signature).unwrap();
        let eth_address =
            <[u8; secp256k1_defs::HASHED_PUBKEY_SERIALIZED_SIZE]>::try_from(eth_address).unwrap();
        let message = Vec::from(message);

        sigs.push(SecpSignature {
            signature,
            recovery_id,
            eth_address,
            message,
        })
    }
    Ok(sigs)
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
        hex::encode(AUTHORIZED_PUBLIC_KEY)
    );

    // If we're using this function for signature verification then we
    // need to check the pubkey is an expected value.
    // Here we are checking the secp256k1 pubkey against a known authorized pubkey.
    if recovered_pubkey.0 != AUTHORIZED_PUBLIC_KEY {
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}
