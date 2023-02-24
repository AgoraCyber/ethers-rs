use ethers_rs::hardhat::cmds::block_run_build;
use ethers_rs::hardhat::colorable::*;
use ethers_rs::hardhat::pretty_env_logger;
fn main() {
    pretty_env_logger::init();

    println!("cargo:rerun-if-changed=sol");

    match block_run_build() {
        Err(err) => {
            println!("{}", err.to_string().red());
        }
        _ => {}
    }
}
