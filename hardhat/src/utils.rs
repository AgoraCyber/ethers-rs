use std::env;
use std::io::ErrorKind;
use std::marker::PhantomData;
use std::path::PathBuf;

use async_process::Child;
use async_process::Command;
use async_process::ExitStatus;

use futures::executor::block_on;
use futures::executor::ThreadPool;
use once_cell::sync::OnceCell;

use crate::error::HardhatError;

/// Kill process and all children processess by process id
#[cfg(target_family = "unix")]
pub async fn kill_process_recursive(process_id: u32) -> anyhow::Result<ExitStatus> {
    let mut child = Command::new("kill")
        .arg(format!("{}", process_id))
        .spawn()?;

    Ok(child.status().await?)
}

/// Returns hardhat base command instance
pub fn hardhat_command<P>(hardhat_root: P) -> anyhow::Result<Command>
where
    P: Into<PathBuf>,
{
    let mut command = Command::new("npx");

    command.arg("hardhat");

    command.current_dir(hardhat_root.into());

    Ok(command)
}

/// Get global thread pool for async task to executing.
pub fn thread_pool() -> &'static ThreadPool {
    static POOLS: OnceCell<ThreadPool> = OnceCell::new();

    POOLS.get_or_init(|| ThreadPool::new().unwrap())
}

/// Returns the hardhat project default path `$CARGO_MANIFEST_DIR/hardhat`
pub fn hardhat_default_path() -> &'static PathBuf {
    static PATH: OnceCell<PathBuf> = OnceCell::new();

    PATH.get_or_init(|| {
        let path: PathBuf = env::current_dir().expect("Get current_dir").into();

        path.join("hardhat")
    })
}

#[async_trait::async_trait]
pub trait HardhatCommandContext {
    /// Execute hardhat command in current_path `hardhat_root`
    ///
    #[allow(unused)]
    fn init_command(hardhat_root: PathBuf, c: &mut Command) -> anyhow::Result<()> {
        Ok(())
    }

    #[allow(unused)]
    async fn start_command(c: &mut Child) -> anyhow::Result<()> {
        Ok(())
    }

    #[allow(unused)]
    async fn drop_command(hardhat_root: PathBuf) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct HardhatCommand<C: HardhatCommandContext> {
    hardhat_root: PathBuf,
    /// hardhat network startup command
    command: Command,
    /// Started hardhat network process
    child_process: Option<Child>,

    _marked: PhantomData<C>,
}

impl<C> HardhatCommand<C>
where
    C: HardhatCommandContext,
{
    pub fn new() -> anyhow::Result<Self> {
        Self::new_with(hardhat_default_path())
    }
    /// Create new hardhat network instance with hardhat project root path.
    pub fn new_with<P>(hardhat_root: P) -> anyhow::Result<Self>
    where
        P: Into<PathBuf>,
    {
        let hardhat_root: PathBuf = hardhat_root.into();

        let mut command = hardhat_command(hardhat_root.clone())?;

        C::init_command(hardhat_root.clone(), &mut command)?;

        Ok(Self {
            hardhat_root,
            child_process: None,
            command,
            _marked: Default::default(),
        })
    }

    /// Returns if network started.
    pub fn is_started(&self) -> bool {
        self.child_process.is_some()
    }

    /// Start hardhat network child process.
    ///
    /// If already started, returns false.
    pub async fn start(&mut self) -> anyhow::Result<bool> {
        if self.is_started() {
            return Ok(false);
        }

        let mut child = match self.command.spawn() {
            Ok(child) => child,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    return Err(HardhatError::NodejsRequired.into());
                } else {
                    return Err(err.into());
                }
            }
        };

        C::start_command(&mut child).await?;

        self.child_process = Some(child);

        return Ok(true);
    }

    /// Stop hardhat network.
    ///
    /// If hardhat network already stopped, returns [`HardhatError::HardhatNetworkStopped`]
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(child_process) = self.child_process.take() {
            kill_process_recursive(child_process.id()).await?;
            Ok(())
        } else {
            Err(HardhatError::HardhatNetworkStopped.into())
        }
    }

    /// Wait until child process stopped and returns status code.
    pub async fn status(&mut self) -> anyhow::Result<ExitStatus> {
        if let Some(mut child_process) = self.child_process.take() {
            Ok(child_process.status().await?)
        } else {
            Err(HardhatError::HardhatNetworkStopped.into())
        }
    }
}

impl<C> Drop for HardhatCommand<C>
where
    C: HardhatCommandContext,
{
    fn drop(&mut self) {
        if self.is_started() {
            let child_process = self.child_process.take().unwrap();
            let hardhat_root = self.hardhat_root.clone();

            block_on(async move {
                let drop_result = C::drop_command(hardhat_root).await;

                log::debug!("drop command result, {:?}", drop_result);

                _ = kill_process_recursive(child_process.id()).await;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir, remove_dir_all};

    use std::path::PathBuf;

    use super::{hardhat_command, hardhat_default_path};

    #[async_std::test]
    async fn start_new_project() {
        let path: PathBuf = hardhat_default_path().clone();
        if path.exists() {
            remove_dir_all(path.clone()).expect("Remove exists path");
        }

        create_dir(path.clone()).expect("Create hardhat dir");

        let mut command = hardhat_command(path)
            .expect("Start hardhat command with current dir $CARGO_MANIFEST_DIR");

        let mut child = command.spawn().expect("Start npx hardhat");

        child.status().await.expect("Wait child process exit.");
    }
}
