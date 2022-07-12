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
use bytemuck::{
    bytes_of,
    Pod,
    Zeroable
};

mod ed25519_defs {
    pub const PUBKEY_SERIALIZED_SIZE: usize = 32;
    pub const SIGNATURE_SERIALIZED_SIZE: usize = 64;
    pub const SIGNATURE_OFFSETS_SERIALIZED_SIZE: usize = 14;
    // bytemuck requires structures to be aligned
    pub const SIGNATURE_OFFSETS_START: usize = 2;
    pub const DATA_START: usize = SIGNATURE_OFFSETS_SERIALIZED_SIZE + SIGNATURE_OFFSETS_START;
    
//    #[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
    #[derive(Default, Debug, Copy, Clone)]
    #[repr(C)]
    pub struct Ed25519SignatureOffsets {
        pub signature_offset: u16, // offset to ed25519 signature of 64 bytes
        pub signature_instruction_index: u16, // instruction index to find signature
        pub public_key_offset: u16, // offset to public key of 32 bytes
        pub public_key_instruction_index: u16, // instruction index to find public key
        pub message_data_offset: u16, // offset to start of message data
        pub message_data_size: u16, // size of message data
        pub message_instruction_index: u16, // index of instruction data to get message data
    }

    pub fn get_ed25519_offsets() -> Ed25519SignatureOffsets {
        let num_signatures: u8 = 1;
        let public_key_offset = DATA_START;
        let signature_offset = public_key_offset.saturating_add(PUBKEY_SERIALIZED_SIZE);
        let message_data_offset = signature_offset.saturating_add(SIGNATURE_SERIALIZED_SIZE);
        let message: &[u8] = b"This is a demo message.";

        Ed25519SignatureOffsets {
            signature_offset: signature_offset as u16,
            signature_instruction_index: u16::MAX,
            public_key_offset: public_key_offset as u16,
            public_key_instruction_index: u16::MAX,
            message_data_offset: message_data_offset as u16,
            message_data_size: message.len() as u16,
            message_instruction_index: u16::MAX,
        }
    }
}

const AUTHORIZED_ED25519_PUBKEY: [u8; PUBLIC_KEY_LENGTH] = [
    211, 210, 72, 176, 173, 140, 129, 224, 36, 99, 29, 4, 141, 117, 74, 94, 173, 213, 199, 210, 26,
    108, 206, 227, 55, 76, 126, 162, 14, 112, 100, 112,
];

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

    let solana_offsets: ed25519_defs::Ed25519SignatureOffsets = ed25519_defs::get_ed25519_offsets();
    msg!("solana offsets: {:#?}", solana_offsets);
//    let instr_offsets: &ed25519_defs::Ed25519SignatureOffsets = bytemuck::try_from_bytes(&ed25519_instr.data[start..end])
//        .map_err(|_| ProgramError::InvalidArgument)?;
//    msg!("instr offsets: {:#?}", instr_offsets);

    let offsets = solana_offsets;
    let pubkey_start = offsets.public_key_offset;
    let pubkey_end = start.saturating_add(PUBKEY_SERIALIZED_SIZE);

    assert!(pubkey_end <= ed25519_instr.data.len());

    let ed25519_instr_pubkey_slice = &ed25519_instr.data[usize::from(pubkey_start)..pubkey_end];
    let ed25519_instr_pubkey = ed25519_dalek::PublicKey::from_bytes(ed25519_instr_pubkey_slice)
        .map_err(|_| ProgramError::InvalidArgument)?;

    msg!("ed25519_instr_pubkey: {:?}", ed25519_instr_pubkey);
    msg!("authorized_pubkey: {:?}", AUTHORIZED_ED25519_PUBKEY);

    if ed25519_instr_pubkey_slice != AUTHORIZED_ED25519_PUBKEY {
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}
