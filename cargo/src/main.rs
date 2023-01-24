use clap::{Parser, Subcommand};
use ethers_hardhat_rs::cmds::{HardhatForceNewProject, HardhatNewProject};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
#[command(author, version, about, long_about = None)]
enum Commands {
    /// Cargo ethers-rs subcommand tools
    Ethers {
        #[command(subcommand)]
        command: Option<Subcommands>,
        #[command(flatten)]
        manifest: clap_cargo::Manifest,
        #[command(flatten)]
        workspace: clap_cargo::Workspace,
        #[command(flatten)]
        features: clap_cargo::Features,
    },
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

    match dispatch_commands().await {
        Err(err) => {
            println!("{}", err.to_string().red())
        }
        _ => {}
    }
}

async fn dispatch_commands() -> anyhow::Result<()> {
    let args = Cli::parse();

    if let Some(Commands::Ethers { command, .. }) = args.command {
        if let Some(command) = command {
            match command {
                Subcommands::Init { force, path } => {
                    exec_new_project(force, path).await?;
                }
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
