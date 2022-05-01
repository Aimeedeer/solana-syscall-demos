use anyhow::Result;
use common::{DemoSecp256k1BasicInstruction, DemoSecp256k1RecoverInstruction};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    keccak,
    signature::{Keypair, Signer},
    transaction::Transaction,
    secp256k1_instruction,
};

/// Basic secp256k1 signature verification using `new_secp256k1_instruction`.
pub fn demo_secp256k1_basic(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let secret_key = libsecp256k1::SecretKey::random(&mut rand::thread_rng());

    // Internally to `new_secp256k1_instruction` and
    // `secp256k_instruction::verify` (the secp256k1 program), this message is
    // keccak-hashed before signing, as in EIP-712.
    let msg = b"hello world";
    let verify_secp256k1_instr =
        secp256k1_instruction::new_secp256k1_instruction(&secret_key, msg);

    let public_key = libsecp256k1::PublicKey::from_secret_key(&secret_key);
    let public_key = secp256k1_instruction::construct_eth_pubkey(&public_key);
    let program_instr = DemoSecp256k1BasicInstruction::build_instruction(
        &program_keypair.pubkey(),
        msg.to_vec(),
        public_key,
    )?;

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

    assert_eq!(signature.len(), secp256k1_instruction::SIGNATURE_SERIALIZED_SIZE);

    let mut public_key_bytes = [0; 64];
    public_key_bytes.copy_from_slice(&public_key.serialize()[1..65]);

    let instr = DemoSecp256k1RecoverInstruction::build_instruction(
        &program_keypair.pubkey(),
        message.to_vec(),
        signature,
        recovery_id.serialize(),
        public_key_bytes,
    );

    todo!()
}
