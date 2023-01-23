use async_process::{Command, Stdio};
use futures::{io::BufReader, AsyncBufReadExt, TryStreamExt};

#[async_std::main]
async fn main() {
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
        println!("{}", line);
    }
}
