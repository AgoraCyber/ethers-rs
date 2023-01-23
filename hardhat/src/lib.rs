#[cfg(test)]
mod tests {
    use async_process::{Command, Stdio};
    use futures::{io::BufReader, AsyncBufReadExt, TryStreamExt};

    #[async_std::test]
    async fn test_start_hardhat_node() {
        _ = pretty_env_logger::try_init();
        let mut child = Command::new("npx")
            .arg("hardhat")
            .arg("node")
            .current_dir("/Users/yayanyang/Workspace/agora/socialdefi-solidity")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Spawn find");

        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();

        while let Some(line) = lines.try_next().await.expect("") {
            if "Any funds sent to them on Mainnet or any other live network WILL BE LOST." == line {
                log::debug!("hardhat node started");
                break;
            }
        }

        let mut child = Command::new("kill")
            .arg(format!("{}", child.id()))
            .spawn()
            .expect("Kill hardhat node");

        log::debug!("{}", child.status().await.expect(""));
    }
}
