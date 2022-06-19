use anyhow::Result;
use common::{
    DemoSecp256k1CustomManyInstruction, DemoSecp256k1RecoverInstruction,
    DemoSecp256k1VerifyBasicInstruction,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    keccak,
    secp256k1_instruction::{
        self,
        SecpSignatureOffsets,
        HASHED_PUBKEY_SERIALIZED_SIZE,
        SIGNATURE_OFFSETS_SERIALIZED_SIZE,
        SIGNATURE_SERIALIZED_SIZE,
    },
    signature::{Keypair, Signer},
    transaction::Transaction,
};


/// The key we'll sign secp256k1 transactions with,
/// and our program will verify.
/// The corresponding pubkey is in the program source.
const AUTHORIZED_SECRET_KEY: [u8; 32] = [
    0x1E, 0xC2, 0xD4, 0x0F, 0x18, 0x08, 0xD7, 0xE7, 0xA3, 0x23, 0x1B, 0xD8, 0x14, 0x7F, 0x24, 0x66,
    0x6B, 0xBB, 0xD3, 0xA1, 0xA2, 0xCF, 0x39, 0xF3, 0x97, 0xF3, 0x05, 0x15, 0xAB, 0x13, 0xCC, 0xC6,
];

/// Basic secp256k1 signature verification using `new_secp256k1_instruction`.
pub fn demo_secp256k1_verify_basic(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let secret_key = libsecp256k1::SecretKey::parse(&AUTHORIZED_SECRET_KEY)?;

    // Internally to `new_secp256k1_instruction` and
    // `secp256k_instruction::verify` (the secp256k1 program), this message is
    // keccak-hashed before signing.
    let msg = b"hello world";
    let secp256k1_instr = secp256k1_instruction::new_secp256k1_instruction(&secret_key, msg);

    let program_instr =
        DemoSecp256k1VerifyBasicInstruction.build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[secp256k1_instr, program_instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}

/// Using the secp256k1 program in a more complex way,
/// without a specific goal.
pub fn demo_secp256k1_custom_many(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    // Sign some messages.
    let mut signatures = vec![];
    for idx in 0..2 {
        let secret_key = libsecp256k1::SecretKey::random(&mut rand::thread_rng());
        let message = format!("hello world {}", idx).into_bytes();
        let message_hash = {
            let mut hasher = keccak::Hasher::default();
            hasher.hash(&message);
            hasher.result()
        };
        let secp_message = libsecp256k1::Message::parse(&message_hash.0);
        let (signature, recovery_id) = libsecp256k1::sign(&secp_message, &secret_key);
        let signature = signature.serialize();
        let recovery_id = recovery_id.serialize();

        let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);
        let eth_address = secp256k1_instruction::construct_eth_pubkey(&public_key);

        let signature_hex = hex::encode(&signature);
        let eth_address_hex = hex::encode(&eth_address);
        let message_hex = hex::encode(&message);

        println!("sig {}: {:?}", idx, signature_hex);
        println!("recid {}: {}", idx, recovery_id);
        println!("eth address {}: {}", idx, eth_address_hex);
        println!("message {}: {}", idx, message_hex);

        signatures.push(SecpSignature {
            signature,
            recovery_id,
            eth_address,
            message,
        });
    }

    let secp256k1_instr_data = make_secp256k1_instruction_data(&signatures, 0)?;
    let secp256k1_instr = Instruction::new_with_bytes(
        solana_sdk::secp256k1_program::ID,
        &secp256k1_instr_data,
        vec![],
    );

    let program_instr =
        DemoSecp256k1CustomManyInstruction.build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[secp256k1_instr, program_instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}

/// A struct to hold the values specified in the `SecpSignatureOffsets` struct.
pub struct SecpSignature {
    pub signature: [u8; SIGNATURE_SERIALIZED_SIZE],
    pub recovery_id: u8,
    pub eth_address: [u8; HASHED_PUBKEY_SERIALIZED_SIZE],
    pub message: Vec<u8>,
}

/// Create the instruction data for a secp256k1 instruction.
///
/// `instruction_index` is the index the secp256k1 instruction will appear
/// within the transaction. For simplicity, this function only supports packing
/// the signatures into the secp256k1 instruction data, and not into any other
/// instructions within the transaction.
fn make_secp256k1_instruction_data(
    signatures: &[SecpSignature],
    instruction_index: u8,
) -> Result<Vec<u8>> {
    assert!(signatures.len() <= u8::max_value().into());

    // We're going to pack all the signatures into the secp256k1 instruction data.
    // Before our signatures though is the signature offset structures
    // the secp256k1 program parses to find those signatures.
    // This value represents the byte offset where the signatures begin.
    let data_start = 1 + signatures.len() * SIGNATURE_OFFSETS_SERIALIZED_SIZE;

    let mut signature_offsets = vec![];
    let mut signature_buffer = vec![];

    for signature_bundle in signatures {
        let data_start = data_start
            .checked_add(signature_buffer.len())
            .expect("overflow");

        let signature_offset = data_start;
        let eth_address_offset = data_start
            .checked_add(SIGNATURE_SERIALIZED_SIZE + 1)
            .expect("overflow");
        let message_data_offset = eth_address_offset
            .checked_add(HASHED_PUBKEY_SERIALIZED_SIZE)
            .expect("overflow");
        let message_data_size = signature_bundle.message.len();

        let signature_offset = u16::try_from(signature_offset)?;
        let eth_address_offset = u16::try_from(eth_address_offset)?;
        let message_data_offset = u16::try_from(message_data_offset)?;
        let message_data_size = u16::try_from(message_data_size)?;

        signature_offsets.push(SecpSignatureOffsets {
            signature_offset,
            signature_instruction_index: instruction_index,
            eth_address_offset,
            eth_address_instruction_index: instruction_index,
            message_data_offset,
            message_data_size,
            message_instruction_index: instruction_index,
        });

        signature_buffer.extend(signature_bundle.signature);
        signature_buffer.push(signature_bundle.recovery_id);
        signature_buffer.extend(&signature_bundle.eth_address);
        signature_buffer.extend(&signature_bundle.message);
    }

    let mut instr_data = vec![];
    instr_data.push(signatures.len() as u8);

    for offsets in signature_offsets {
        let offsets = bincode::serialize(&offsets)?;
        instr_data.extend(offsets);
    }

    instr_data.extend(signature_buffer);

    Ok(instr_data)
}

/// Using the `secp256k1_recover` function (`sol_secp256k1_recover` syscall) to
/// recover a public key from a 32-byte message (a keccak hash), a 64-byte
/// signature, and recovery id.
pub fn demo_secp256k1_recover(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let secret_key = libsecp256k1::SecretKey::parse(&AUTHORIZED_SECRET_KEY)?;

    let message = b"hello world";
    let message_hash = {
        let mut hasher = keccak::Hasher::default();
        hasher.hash(message);
        hasher.result()
    };

    let secp_message = libsecp256k1::Message::parse(&message_hash.0);
    let (signature, recovery_id) = libsecp256k1::sign(&secp_message, &secret_key);

    let signature = signature.serialize();

    assert_eq!(
        signature.len(),
        SIGNATURE_SERIALIZED_SIZE
    );

    let instr = DemoSecp256k1RecoverInstruction {
        message: message.to_vec(),
        signature,
        recovery_id: recovery_id.serialize(),
    }
    .build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}

#[allow(unused)]
pub fn test_libsecp256k1_malleability() -> Result<()> {
    let secret_key = libsecp256k1::SecretKey::random(&mut rand::thread_rng());
    let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);

    let message = b"hello world";
    let message_hash = {
        let mut hasher = keccak::Hasher::default();
        hasher.hash(message);
        hasher.result()
    };

    let secp_message = libsecp256k1::Message::parse(&message_hash.0);
    let (signature, recovery_id) = libsecp256k1::sign(&secp_message, &secret_key);

    let signature = signature.serialize();
    let signature = libsecp256k1::Signature::parse_standard_slice(&signature)?;

    println!("pubser: {:#04X?}", public_key.serialize());
    println!("sigser: {:#04X?}", signature.serialize());

    println!("sig: {:?}", signature);
    println!("recid: {:?}", recovery_id);

    let recovered_key = libsecp256k1::recover(&secp_message, &signature, &recovery_id)?;

    println!("{:?}", public_key);
    println!("{:?}", recovered_key);
    assert_eq!(public_key, recovered_key);

    let verified = libsecp256k1::verify(&secp_message, &signature, &public_key);
    println!("verified: {}", verified);

    println!("---");

    let mut signature = signature;
    signature.s = -signature.s;
    let recovery_id = libsecp256k1::RecoveryId::parse(recovery_id.serialize() ^ 1)?;

    let signature = signature.serialize();
    let signature = libsecp256k1::Signature::parse_standard_slice(&signature)?;

    println!("sig: {:?}", signature);
    println!("recid: {:?}", recovery_id);

    let recovered_key = libsecp256k1::recover(&secp_message, &signature, &recovery_id)?;

    println!("{:?}", public_key);
    println!("{:?}", recovered_key);
    assert_eq!(public_key, recovered_key);

    let verified = libsecp256k1::verify(&secp_message, &signature, &public_key);
    println!("verified: {}", verified);

    Ok(())
}
