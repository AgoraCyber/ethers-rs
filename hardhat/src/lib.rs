pub mod cmds;
pub mod error;
pub mod utils;

pub use colorable;
pub use futures;
pub use pretty_env_logger;

#[macro_export]
macro_rules! hardhat_network {
    () => {
        $crate::cmds::HardhatNetwork::new_with(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sol"),
        )
        .expect("Start hardhat network")
    };
}
