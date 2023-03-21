use rusqlite::OpenFlags;

use crate::cli::{Args, BurnsArgs, Network};

pub fn burns(args: Args, burns_args: BurnsArgs) {
    let mode = match args.network {
        Network::Mainnet => "mainnet/",
        Network::Testnet => "xenon/",
    };
    let db_file = args.db_dir.join(mode).join("burnchain/burnchain.sqlite");
    let conn =
        rusqlite::Connection::open_with_flags(db_file, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();

    let mut statement = conn
        .prepare(r#"
            SELECT JSON_EXTRACT(op, "$.LeaderBlockCommit.block_height") as block_height, JSON_EXTRACT(op, "$.LeaderBlockCommit.burn_fee") as burn_fee
            FROM burnchain_db_block_ops
            ORDER BY block_height DESC
        "#,)
        .unwrap();

    let mut rows = statement.query::<[u8; 0]>([]).unwrap();

    let mut height_fee_pairs: Vec<(u64, u64)> = vec![];

    while let Some(row) = rows.next().unwrap() {
        let Some(height) = row.get::<_, Option<i64>>(0).unwrap() else { continue };
        let Some(fee) = row.get::<_, Option<i64>>(1).unwrap() else { continue };

        height_fee_pairs.push((height as u64, fee as u64));
    }

    let last_block = height_fee_pairs.first().unwrap().0;
    let cutoff_block = last_block - burns_args.blocks;

    let mut burn_fees: Vec<u64> = height_fee_pairs
        .into_iter()
        .filter(|(height, _)| *height >= cutoff_block)
        .map(|(_, fee)| fee)
        .collect();

    burn_fees.sort();

    println!(
        "Burn stats for last {} blocks: min={} max={} mean={} avg={}",
        burns_args.blocks,
        burn_fees.first().unwrap(),
        burn_fees.last().unwrap(),
        burn_fees[burn_fees.len() / 2],
        burn_fees.iter().sum::<u64>() / burn_fees.len() as u64
    );
}
