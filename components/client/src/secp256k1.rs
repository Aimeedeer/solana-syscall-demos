use anyhow::Result;
use common::{DemoSecp256k1RecoverInstruction, DemoSecp256k1VerifyBasicInstruction};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    keccak, secp256k1_instruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

/// Basic secp256k1 signature verification using `new_secp256k1_instruction`.
pub fn demo_secp256k1_verify_basic(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let secret_key = libsecp256k1::SecretKey::random(&mut rand::thread_rng());

    // Internally to `new_secp256k1_instruction` and
    // `secp256k_instruction::verify` (the secp256k1 program), this message is
    // keccak-hashed before signing.
    let msg = b"hello world";
    let verify_secp256k1_instr = secp256k1_instruction::new_secp256k1_instruction(&secret_key, msg);

    let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);
    let public_key = secp256k1_instruction::construct_eth_pubkey(&public_key);
    let program_instr = DemoSecp256k1VerifyBasicInstruction {
    }
    .build_instruction(&program_keypair.pubkey());

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[verify_secp256k1_instr, program_instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}

/// Using the `secp256k1_recover` function (`sol_secp256k1_recover` syscall) to
/// recover a public key from a 32-byte message (a keccak hash), a 64-byte
/// signature, and recovery id.
pub fn demo_secp256k1_recover(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
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

    assert_eq!(
        signature.len(),
        secp256k1_instruction::SIGNATURE_SERIALIZED_SIZE
    );

    let mut public_key_bytes = [0; 64];
    public_key_bytes.copy_from_slice(&public_key.serialize()[1..65]);

    let instr = DemoSecp256k1RecoverInstruction {
        message: message.to_vec(),
        signature,
        recovery_id: recovery_id.serialize(),
        expected_signer_pubkey: public_key_bytes,
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

pub fn test_libsecp256k1_malleability() -> Result<()> {
    use solana_sdk::keccak;

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
