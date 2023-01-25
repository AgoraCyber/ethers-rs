use ethers_hardhat_rs::cmds::block_run_build;
use ethers_hardhat_rs::colorable::*;
use ethers_hardhat_rs::pretty_env_logger;
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
