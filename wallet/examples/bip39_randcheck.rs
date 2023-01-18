use ethers_wallet_rs::hd_wallet::bip39::{languages, Bip39Generator};
use rdbc::Database;
use rdbc::*;

async fn fetch_count(db: &mut Database) -> anyhow::Result<i64> {
    let mut rows = db
        .prepare("SELECT count(id) FROM mnemonic")
        .await
        .expect("Get generated count")
        .query(vec![])
        .await
        .expect("Get generated count");

    assert!(rows.next().await.expect("Next"));

    let count = rows
        .get(0, ColumnType::I64)
        .await
        .expect("Get count result")
        .expect("Get count");

    if let ArgValue::I64(value) = count {
        Ok(value)
    } else {
        Err(anyhow::format_err!("Get mnemonic count failed"))
    }
}

async fn create_db_if_not_exists(db: &mut Database) -> anyhow::Result<()> {
    db.prepare(
        "CREATE TABLE IF NOT EXISTS mnemonic(id INTEGER PRIMARY KEY ASC, content TEXT UNIQUE)",
    )
    .await
    .expect("Create table if not exists")
    .execute(vec![])
    .await
    .expect("Create table i f not exists");

    Ok(())
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    rdbc_sqlite3::register_sqlite3().expect("Register sqlite3 database");

    let mut db: Database = open("sqlite3", "file:./bip32.db").unwrap();

    create_db_if_not_exists(&mut db).await?;

    let mut count = fetch_count(&mut db).await?;

    log::info!("Start new bip39 rand test, previously created {}", count);

    let gen = Bip39Generator::new(languages::en_us());

    let mut stmt = db
        .prepare("INSERT INTO mnemonic(content) VALUES(:m)")
        .await
        .expect("Create table if not exists");

    loop {
        let mnemonic = gen.gen_mnemonic::<16>().expect("Gen mnemonic");

        assert!(
            gen.mnemonic_check(&mnemonic).is_ok(),
            "Test checksum for generated mnemonic"
        );

        stmt.execute(vec![rdbc::Argument {
            name: rdbc::ArgName::String(":m".to_owned()),
            value: driver::ArgValue::String(mnemonic.clone()),
        }])
        .await
        .expect(&format!("Insert mnemonic {}", mnemonic));

        count += 1;

        log::info!("Create mnemonic {},{}", count, mnemonic);
    }
}
