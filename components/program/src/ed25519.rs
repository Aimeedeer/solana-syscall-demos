use common::DemoEd25519Instruction;
use ed25519_dalek::PUBLIC_KEY_LENGTH;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    ed25519_program,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    sysvar,
};

mod ed25519_defs {
    use solana_program::program_error::ProgramError;

    pub const PUBKEY_SERIALIZED_SIZE: usize = 32;
    pub const SIGNATURE_SERIALIZED_SIZE: usize = 64;
    pub const SIGNATURE_OFFSETS_SERIALIZED_SIZE: usize = 14;
    // bytemuck requires structures to be aligned
    pub const SIGNATURE_OFFSETS_START: usize = 2;
    pub const DATA_START: usize = SIGNATURE_OFFSETS_SERIALIZED_SIZE + SIGNATURE_OFFSETS_START;

    #[derive(Default, Debug, Copy, Clone)]
    pub struct Ed25519SignatureOffsets {
        pub signature_offset: u16, // offset to ed25519 signature of 64 bytes
        pub signature_instruction_index: u16, // instruction index to find signature
        pub public_key_offset: u16, // offset to public key of 32 bytes
        pub public_key_instruction_index: u16, // instruction index to find public key
        pub message_data_offset: u16, // offset to start of message data
        pub message_data_size: u16, // size of message data
        pub message_instruction_index: u16, // index of instruction data to get message data
    }

    pub fn iter_signature_offsets(
        ed25519_instr_data: &[u8],
    ) -> Result<Ed25519SignatureOffsets, ProgramError> {
        // First element is the number of num_signatures
        let num_signature = *ed25519_instr_data
            .get(0)
            .ok_or(ProgramError::InvalidArgument)?;

        let public_key_offset = DATA_START;
        let signature_offset = public_key_offset.saturating_add(PUBKEY_SERIALIZED_SIZE);
        let message_data_offset = signature_offset.saturating_add(SIGNATURE_SERIALIZED_SIZE);

        fn decode_u16(chunk: &[u8], index: usize) -> u16 {
            u16::from_le_bytes(<[u8; 2]>::try_from(&chunk[index..index + 2]).unwrap())
        }

        let message_data_size = u16::from_le_bytes(
            <[u8; 2]>::try_from(&ed25519_instr_data[message_data_offset..]).unwrap(),
        );

        Ok(Ed25519SignatureOffsets {
            signature_offset: signature_offset as u16,
            signature_instruction_index: u16::MAX,
            public_key_offset: public_key_offset as u16,
            public_key_instruction_index: u16::MAX,
            message_data_offset: message_data_offset as u16,
            message_data_size: message_data_size as u16,
            message_instruction_index: u16::MAX,
        })
    }
}

const AUTHORIZED_ED25519_PUBKEY: [u8; PUBLIC_KEY_LENGTH] = [
    211, 210, 72, 176, 173, 140, 129, 224, 36, 99, 29, 4, 141, 117, 74, 94, 173, 213, 199, 210, 26,
    108, 206, 227, 55, 76, 126, 162, 14, 112, 100, 112,
];

const EXPECTED_MESSAGE: &[u8] = b"This is a demo message.";

pub fn demo_ed25519(
    _instruction: DemoEd25519Instruction,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("demo ed25519");

    use ed25519_defs::*;
    let account_info_iter = &mut accounts.iter();

    // The instructions sysvar gives access to the instructions in the transaction.
    let instructions_sysvar_account = next_account_info(account_info_iter)?;
    assert!(sysvar::instructions::check_id(
        instructions_sysvar_account.key
    ));

    let ed25519_instr =
        sysvar::instructions::load_instruction_at_checked(0, instructions_sysvar_account)?;

    assert!(ed25519_program::check_id(&ed25519_instr.program_id));
    assert!(ed25519_instr.data.len() > 1);

    let num_signatures = ed25519_instr.data[0];
    assert_eq!(1, num_signatures);

    // ?? for i in 0..num_signatures {
    let start = usize::from(num_signatures)
        .saturating_mul(SIGNATURE_OFFSETS_SERIALIZED_SIZE)
        .saturating_add(SIGNATURE_OFFSETS_START);
    let end = start.saturating_add(SIGNATURE_OFFSETS_SERIALIZED_SIZE);

    let offsets = ed25519_defs::iter_signature_offsets(&ed25519_instr.data)?;
    msg!("offsets: {:#?}", offsets);

    let pubkey_start = usize::from(offsets.public_key_offset);
    let pubkey_end = pubkey_start.saturating_add(PUBKEY_SERIALIZED_SIZE);
    let ed25519_instr_pubkey_slice = &ed25519_instr.data[pubkey_start..pubkey_end];
    let ed25519_instr_pubkey = ed25519_dalek::PublicKey::from_bytes(ed25519_instr_pubkey_slice)
        .map_err(|_| ProgramError::InvalidArgument)?;

    msg!("ed25519_instr_pubkey: {:?}", ed25519_instr_pubkey);
    msg!("authorized_pubkey: {:?}", AUTHORIZED_ED25519_PUBKEY);

    if ed25519_instr_pubkey_slice != AUTHORIZED_ED25519_PUBKEY {
        return Err(ProgramError::InvalidArgument);
    }

    let expected_message = EXPECTED_MESSAGE;
    assert_eq!(
        usize::from(offsets.message_data_size),
        expected_message.len()
    );

    let msg_start = usize::from(offsets.message_data_offset);
    let msg_end = msg_start.saturating_add(usize::from(offsets.message_data_size));
    let ed25519_instr_message = &ed25519_instr.data[msg_start..msg_end];

    msg!("ed25519_instr_message: {:?}", ed25519_instr_message);
    msg!("expected_message: {:?}", expected_message);

    Ok(())
}
