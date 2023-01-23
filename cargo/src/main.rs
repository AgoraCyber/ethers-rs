use clap::{Parser, Subcommand};

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
        /// Use a library template to generating `ethers-rs` crate.
        #[arg(short, long, default_value_t = false)]
        lib: bool,

        /// Name of the crate being created
        name: String,
    },
}

fn main() {
    let args = Cli::parse();
    println!("subcommand ... {:#?}", args);
}
