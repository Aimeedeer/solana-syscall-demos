use anyhow::{anyhow, bail, Context, Result};
use log::info;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair, Signer},
};

mod system_test;
mod sysvar_test;

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("solana_client::rpc_client", log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let config = load_config()?;
    let client = connect(&config)?;
    let version = client.get_version()?;
    info!("version: {}", version);

    let program_keypair = get_program_keypair(&client)?;
    let program_id = program_keypair.pubkey();
    println!("program id: {:#?}", program_id);

    {
        // Solana's system_instruction
        let payer = &config.keypair;

        {
            // program tests
            let new_account_for_program_test = Keypair::new();

            // program test: create_account
            system_test::create_account_via_program(
                &client,
                &program_id,
                payer,
                &new_account_for_program_test,
            )?;

            // program test: allocate
            let allocate_space = 1024;
            system_test::allocate_via_program(
                &client,
                &program_id,
                payer,
                &new_account_for_program_test,
                allocate_space,
            )?;

            // program test: transfer
            let amount = 1_000_000;
            system_test::transfer_via_program(
                &client,
                &program_id,
                payer,
                &new_account_for_program_test.pubkey(),
                amount,
            )?;

            // program test: transfer_many
            {
                let another_test_account = Keypair::new();
                system_test::create_account_via_program(
                    &client,
                    &program_id,
                    payer,
                    &another_test_account,
                )?;

                let to_and_amount = vec![
                    (new_account_for_program_test.pubkey(), 1_000_000),
                    (another_test_account.pubkey(), 1_000),
                ];

                system_test::transfer_many_via_program(
                    &client,
                    &program_id,
                    payer,
                    &to_and_amount,
                )?;
            }

            // rpc tests
            let new_account_for_rpc_test = Keypair::new();

            // rpc test: create_account
            system_test::create_account_via_rpc(&client, payer, &new_account_for_rpc_test)?;

            // rpc test: allocate
            let allocate_space = 2048;
            system_test::allocate_via_rpc(
                &client,
                payer,
                &new_account_for_rpc_test,
                allocate_space,
            )?;

            // rpc test: transfer
            let amount = 1_000_000;
            system_test::transfer_via_rpc(
                &client,
                payer,
                &new_account_for_rpc_test.pubkey(),
                amount,
            )?;

            // rpc test: transfer_many
            {
                let another_test_account = Keypair::new();
                system_test::create_account_via_rpc(&client, payer, &another_test_account)?;

                let to_and_amount = vec![
                    (new_account_for_rpc_test.pubkey(), 1_000_000),
                    (another_test_account.pubkey(), 1_000),
                ];

                system_test::transfer_many_via_rpc(&client, payer, &to_and_amount)?;
            }
        }
    }
    /*
        {
            // Solana's sysvar
            sysvar_test::sysvar_printing_via_program(&client, &program_id, &config.keypair)?;
            sysvar_test::sysvar_printing_via_rpc(&client)?;
        }
    */
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

fn get_program_keypair(client: &RpcClient) -> Result<Keypair> {
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
