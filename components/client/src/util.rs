use anyhow::{anyhow, bail, Context, Result};
use log::info;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair, Signer},
};

static DEPLOY_PATH: &str = "target/deploy";
static PROGRAM_KEYPAIR_PATH: &str = "program-keypair.json";

pub struct Config {
    pub json_rpc_url: String,
    pub websocket_url: String,
    pub keypair: Keypair,
}

pub fn load_config(rpc_url: Option<String>) -> Result<Config> {
    let config_file = solana_cli_config::CONFIG_FILE
        .as_ref()
        .ok_or_else(|| anyhow!("config file path"))?;
    let cli_config = solana_cli_config::Config::load(config_file)?;
    let json_rpc_url = rpc_url
        .clone()
        .map(|url| solana_clap_utils::input_validators::normalize_to_url_if_moniker(url))
        .unwrap_or(cli_config.json_rpc_url.clone());
    let websocket_url = rpc_url
        .map(|url| solana_clap_utils::input_validators::normalize_to_url_if_moniker(url))
        .map(|url| solana_cli_config::Config::compute_websocket_url(&url))
        .unwrap_or(cli_config.websocket_url);
    let websocket_url = if !websocket_url.is_empty() {
        websocket_url
    } else {
        solana_cli_config::Config::compute_websocket_url(&cli_config.json_rpc_url)
    };

    let keypair = read_keypair_file(&cli_config.keypair_path).map_err(|e| anyhow!("{}", e))?;

    Ok(Config {
        json_rpc_url,
        websocket_url,
        keypair,
    })
}

pub fn connect(config: &Config) -> Result<RpcClient> {
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
