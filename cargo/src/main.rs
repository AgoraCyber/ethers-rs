use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use ethers_hardhat_rs::cmds::{HardhatForceNewProject, HardhatNewProject};

#[derive(Debug, Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum Cargo {
    Ethers(Ethers),
}

#[derive(Debug, clap::Args)]
#[command(author, version, about, long_about = None)]
struct Ethers {
    #[command(subcommand)]
    command: Option<Subcommands>,
    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    #[command(author, version, about, long_about = None)]
    Init {
        /// Force the creation of new hardhat project
        #[arg(short, long, default_value_t = false)]
        force: bool,

        #[arg(short, long)]
        path: Option<String>,
    },
}

#[async_std::main]
async fn main() {
    use colorable::*;

    // pretty_env_logger::init();

    match dispatch_commands().await {
        Err(err) => {
            println!("{}", err.to_string().red())
        }
        _ => {}
    }
}

async fn dispatch_commands() -> anyhow::Result<()> {
    let Cargo::Ethers(ethers) = Cargo::parse();

    env_logger::Builder::new()
        .filter_level(ethers.verbose.log_level_filter())
        .init();

    if let Some(command) = ethers.command {
        match command {
            Subcommands::Init { force, path } => {
                exec_new_project(force, path).await?;
            }
        }
    }

    Ok(())
}

async fn exec_new_project(force: bool, path: Option<String>) -> anyhow::Result<()> {
    if force {
        let mut command = if let Some(path) = path {
            HardhatForceNewProject::new_with(path)?
        } else {
            HardhatForceNewProject::new()?
        };

        command.start().await?;

        command.status().await?;
    } else {
        let mut command = if let Some(path) = path {
            HardhatNewProject::new_with(path)?
        } else {
            HardhatNewProject::new()?
        };

        command.start().await?;

        command.status().await?;
    };

    Ok(())
}
