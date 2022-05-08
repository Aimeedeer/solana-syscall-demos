use anyhow::Result;
use log::info;
use solana_sdk::signature::Signer;
use clap::Parser;
mod secp256k1;
mod sysvars;
mod util;

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    command: Command,
    #[clap(long, global = true)]
    url: Option<String>,
}

#[derive(clap::Subcommand)]
enum Command {
    PrintSysvarsViaProgram,
    PrintSysvarsViaClient,
    DemoSecp256k1VerifyBasic,
    DemoSecp256k1Recover,
}

fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("solana_client", log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let opts = Opts::parse();
    let command = opts.command;

    let config = util::load_config(opts.url)?;
    let client = util::connect(&config)?;
    let version = client.get_version()?;
    info!("version: {}", version);

    let program_keypair = util::get_program_keypair(&client)?;
    println!("program id: {:#?}", program_keypair.pubkey());

    match command {
        Command::PrintSysvarsViaProgram => {
            sysvars::print_sysvars_via_program(&config, &client, &program_keypair)?;
        }
        Command::PrintSysvarsViaClient => {
            sysvars::print_sysvars_via_client(&client)?;
        }
        Command::DemoSecp256k1VerifyBasic => {
            secp256k1::demo_secp256k1_verify_basic(&config, &client, &program_keypair)?;
        }
        Command::DemoSecp256k1Recover => {
            secp256k1::demo_secp256k1_recover(&config, &client, &program_keypair)?;
        }
    }

    Ok(())
}
