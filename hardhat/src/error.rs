use async_process::ExitStatus;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HardhatError {
    #[error("Hardhat project path is not exists or is not directory, {0}")]
    HardhatDirectory(String),

    #[error("Hardhat network already started")]
    HardhatNetworkStarted,

    #[error("Hardhat network already stopped")]
    HardhatNetworkStopped,

    #[error("Child process '{0}' {1}")]
    ChildProcess(String, ExitStatus),

    #[error("Hardhat project directory exists, {0}")]
    ProjectExists(String),
    #[error("Nodejs env not found")]
    NodejsRequired,
}
