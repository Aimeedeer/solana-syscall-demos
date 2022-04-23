use anyhow::Result;
use bincode;
use common::PrintSysvarsInstruction;
use log::info;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use clap::Parser;

mod util;

#[derive(Parser)]
enum Command {
    PrintSysvarsViaProgram,
    PrintSysvarsViaClient,
    DemoSecp256k1,
}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("solana_client", log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let command = Command::parse();

    let config = util::load_config()?;
    let client = util::connect(&config)?;
    let version = client.get_version()?;
    info!("version: {}", version);

    let program_keypair = util::get_program_keypair(&client)?;
    println!("program id: {:#?}", program_keypair.pubkey());

    match command {
        Command::PrintSysvarsViaProgram => {
            print_sysvars_via_program(&config, &client, &program_keypair)?;
        }
        Command::PrintSysvarsViaClient => {
            print_sysvars_via_client(&client)?;
        }
        Command::DemoSecp256k1 => {
            demo_secp256k1(&config, &client, &program_keypair)?;
        }
    }

    Ok(())
}

fn print_sysvars_via_program(
    config: &util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let instr = PrintSysvarsInstruction::build_instruction(
        &config.keypair.pubkey(),
        &program_keypair.pubkey(),
    )?;

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

fn print_sysvars_via_client(
    client: &RpcClient,
) -> Result<()> {
    println!("--------------------------------------- sysvar client printing ---------------------------------------");

    use solana_sdk::sysvar::{
        clock, clock::Clock, epoch_schedule, epoch_schedule::EpochSchedule, instructions, rent,
        rent::Rent, slot_hashes, slot_hashes::SlotHashes, slot_history,
        slot_history::SlotHistory, stake_history, stake_history::StakeHistory, Sysvar,
    };

    let sysvar_program_id = clock::ID;
    println!("clock::ID: {}", sysvar_program_id);
    println!("clock::check_id: {}", clock::check_id(&sysvar_program_id));
    println!("clock::Clock::size_of: {}", Clock::size_of());

    let account = client.get_account(&sysvar_program_id)?;
    println!("clock account: {:#?}", account);
    let data: Clock = bincode::deserialize(&account.data)?;
    println!("clock account data: {:#?}", data);

    let sysvar_program_id = epoch_schedule::ID;
    println!("epoch_schedule::ID: {}", sysvar_program_id);
    println!(
        "epoch_schedule::check_id: {}",
        epoch_schedule::check_id(&sysvar_program_id)
    );
    println!(
        "epoch_schedule::EpochSchedule::size_of: {}",
        EpochSchedule::size_of()
    );

    let account = client.get_account(&sysvar_program_id)?;
    println!("epoch_schedule account: {:#?}", account);
    let data: EpochSchedule = bincode::deserialize(&account.data)?;
    println!("epoch_schedule account data: {:#?}", data);

    let sysvar_program_id = instructions::ID;
    println!("instructions::ID: {}", sysvar_program_id);
    println!(
        "instructions::check_id: {}",
        instructions::check_id(&sysvar_program_id)
    );

    let sysvar_program_id = rent::ID;
    println!("rent::ID: {}", sysvar_program_id);
    println!("rent::check_id: {}", rent::check_id(&sysvar_program_id));
    println!("rent::Rent::size_of: {}", Rent::size_of());

    let account = client.get_account(&sysvar_program_id)?;
    println!("rent account: {:#?}", account);
    let data: Rent = bincode::deserialize(&account.data)?;
    println!("rent account data: {:?}", data);

    let sysvar_program_id = slot_hashes::ID;
    println!("slot_hashes::ID: {}", sysvar_program_id);
    println!(
        "slot_hashes::check_id: {}",
        slot_hashes::check_id(&sysvar_program_id)
    );
    println!(
        "slot_hashes::SlotHashes::size_of: {}",
        SlotHashes::size_of()
    );

    let account = client.get_account(&sysvar_program_id)?;
    println!("slot_hashes account: {:#?}", account);
    let data: SlotHashes = bincode::deserialize(&account.data)?;
    println!("slot_hashes account data: {:?}", data);

    let sysvar_program_id = slot_history::ID;
    println!("slot_history::ID: {}", sysvar_program_id);
    println!(
        "slot_history::check_id: {}",
        slot_history::check_id(&sysvar_program_id)
    );
    println!(
        "slot_history::SlotHistory::size_of: {}",
        SlotHistory::size_of()
    );

    let account = client.get_account(&sysvar_program_id)?;
    println!("slot_history account: {:#?}", account);
    let data: SlotHistory = bincode::deserialize(&account.data)?;
    println!("slot_history account data: {:?}", data);

    let sysvar_program_id = stake_history::ID;
    println!("stake_history::ID: {}", sysvar_program_id);
    println!(
        "stake_history::check_id: {}",
        stake_history::check_id(&sysvar_program_id)
    );
    println!(
        "stake_history::StakeHistory::size_of: {}",
        StakeHistory::size_of()
    );
    let account = client.get_account(&sysvar_program_id)?;
    println!("stake_history account: {:#?}", account);

    let data: StakeHistory = bincode::deserialize(&account.data)?;
    println!("stake_history account data: {:#?}", data);

    Ok(())
}

fn demo_secp256k1(
    config: &util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    todo!()
}
