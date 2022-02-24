use anyhow::{anyhow, bail, Context, Result};
use bincode;
use common::CustomInstruction;
use log::info;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};

fn main() -> Result<()> {
    let config = load_config()?;
    let client = connect(&config)?;
    let version = client.get_version()?;

    let program_keypair = get_program_keypair(&client)?;
    println!("program id: {:#?}", program_keypair.pubkey());

    env_logger::init();

    // sysvar printing via program
    {
        let instr = CustomInstruction::build_instruction(
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
    }

    // sysvar printing via client
    {
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

        // print `()`
        // account.deserialize_data()
    }

    Ok(())
}

static DEPLOY_PATH: &str = "target/deploy";
static PROGRAM_KEYPAIR_PATH: &str = "program-keypair.json";

pub struct Config {
    json_rpc_url: String,
    keypair: Keypair,
}

fn load_config() -> Result<Config> {
    let config_file = solana_cli_config::CONFIG_FILE
        .as_ref()
        .ok_or_else(|| anyhow!("config file path"))?;
    let cli_config = solana_cli_config::Config::load(config_file)?;
    let json_rpc_url = cli_config.json_rpc_url;
    let keypair = read_keypair_file(&cli_config.keypair_path).map_err(|e| anyhow!("{}", e))?;

    Ok(Config {
        json_rpc_url,
        keypair,
    })
}

fn connect(config: &Config) -> Result<RpcClient> {
    info!("connecting to solana node at {}", config.json_rpc_url);
    let client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), CommitmentConfig::confirmed());

    let version = client.get_version()?;
    info!("RPC version: {:?}", version);

    Ok(client)
}

pub fn get_program_keypair(client: &RpcClient) -> Result<Keypair> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let deploy_path = format!("{}/../../{}", manifest_dir, DEPLOY_PATH);
    let program_keypair_path = format!("{}/{}", deploy_path, PROGRAM_KEYPAIR_PATH);

    info!("loading program keypair from {}", program_keypair_path);

    let program_keypair = read_keypair_file(&program_keypair_path)
        .map_err(|e| anyhow!("{}", e))
        .context("unable to load program keypair")?;

    let program_id = program_keypair.pubkey();

    info!("program id: {}", program_id);

    let account = client
        .get_account(&program_id)
        .context("unable to get program account")?;

    info!("program account: {:?}", account);

    if !account.executable {
        bail!("solana account not executable");
    }

    Ok(program_keypair)
}
