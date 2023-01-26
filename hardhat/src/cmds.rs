use std::{
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
};

use async_process::{Child, Command, ExitStatus, Stdio};
use futures::{executor::block_on, io::BufReader, task::SpawnExt, AsyncBufReadExt, TryStreamExt};

use crate::{
    error::HardhatError,
    utils::{thread_pool, HardhatCommand, HardhatCommandContext},
};

/// Hardhat network helper structure.
///
#[derive(Debug)]
pub struct NetworkContext;

#[async_trait::async_trait]
impl HardhatCommandContext for NetworkContext {
    fn init_command(_: PathBuf, c: &mut Command) -> anyhow::Result<()> {
        c.arg("node").stdout(Stdio::piped()).stderr(Stdio::piped());

        Ok(())
    }

    async fn start_command(child_process: &mut Child) -> anyhow::Result<()> {
        let mut lines = BufReader::new(child_process.stdout.take().unwrap()).lines();

        let mut waiting_started = false;

        while let Some(line) = lines.try_next().await.expect("") {
            if "Any funds sent to them on Mainnet or any other live network WILL BE LOST." == line {
                log::info!("hardhat node started");
                waiting_started = true;
                break;
            }
        }

        // Start hardhat network failed.
        if !waiting_started {
            let status = child_process.status().await?;

            return Err(HardhatError::ChildProcess("npx hardhat node".to_string(), status).into());
        }

        thread_pool().spawn(async move {
            while let Some(line) = lines.try_next().await.expect("") {
                log::trace!(target:"hardhat node" ,"{}", line);
            }
        })?;

        Ok(())
    }
}
/// Command helper structure for hardhat network
pub type HardhatNetwork = HardhatCommand<NetworkContext>;

/// Create new hardhat project if target not exists
#[derive(Debug)]
pub struct NewProjectContext;

#[async_trait::async_trait]
impl HardhatCommandContext for NewProjectContext {
    fn init_command(hardhat_root: PathBuf, _c: &mut Command) -> anyhow::Result<()> {
        if hardhat_root.exists() {
            Err(HardhatError::ProjectExists(hardhat_root.to_string_lossy().into_owned()).into())
        } else {
            log::debug!(
                "create hardhat project root dir,{}",
                hardhat_root.to_string_lossy()
            );

            create_dir_all(hardhat_root)?;

            Ok(())
        }
    }
}

/// Command for creating new hardhat project .
pub type HardhatNewProject = HardhatCommand<NewProjectContext>;

/// Create new hardhat project, even if target path exists.
///
/// If target path exists, remove it first.
#[derive(Debug)]
pub struct ForceNewProjectContext;

#[async_trait::async_trait]
impl HardhatCommandContext for ForceNewProjectContext {
    fn init_command(hardhat_root: PathBuf, _c: &mut Command) -> anyhow::Result<()> {
        if hardhat_root.exists() {
            remove_dir_all(hardhat_root.clone())?;
        }

        create_dir_all(hardhat_root)?;

        Ok(())
    }
}

/// Command for creating new hardhat project .
pub type HardhatForceNewProject = HardhatCommand<ForceNewProjectContext>;

#[derive(Debug)]
pub struct BuildProjectContext;

#[async_trait::async_trait]
impl HardhatCommandContext for BuildProjectContext {
    fn init_command(hardhat_root: PathBuf, c: &mut Command) -> anyhow::Result<()> {
        log::debug!(
            "try build hardhat project {}",
            hardhat_root.to_string_lossy()
        );

        c.arg("compile");

        Ok(())
    }
}

/// Command for creating new hardhat project .
pub type HardhatBuildProject = HardhatCommand<BuildProjectContext>;

/// Helper fn to block run [`HardhatBuildProject`] command
pub fn block_run_build() -> anyhow::Result<ExitStatus> {
    block_on(async {
        let mut command = HardhatBuildProject::new()?;

        command.start().await?;

        Ok(command.status().await?)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn test_start_network() {
        _ = pretty_env_logger::try_init();

        let mut network =
            HardhatNetwork::new().expect("Create default hardhat network helper command");

        assert!(!network.is_started());

        network.start().await.expect("Start hardhat network");
    }

    #[async_std::test]
    async fn test_create_new_project() {
        _ = pretty_env_logger::try_init();

        let mut command =
            HardhatForceNewProject::new().expect("Create default hardhat new project command");

        assert!(!command.is_started());

        command
            .start()
            .await
            .expect("Start create new hardhat project");

        assert!(command
            .status()
            .await
            .expect("Create new project")
            .success());

        _ = HardhatNewProject::new().expect_err("Create new project in exists path, expect failed");
    }
}
