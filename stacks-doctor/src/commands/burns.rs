use std::ascii::AsciiExt;

use reqwest::blocking::Client;
use serde_json::Value;

use crate::cli::BurnsArgs;

pub fn burns(args: BurnsArgs) {
    let db_file = args.db_dir.join("xenon/burnchain/burnchain.sqlite");
    let conn = rusqlite::Connection::open(db_file).unwrap();

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
    let cutoff_block = last_block - args.blocks;

    let mut burn_fees: Vec<u64> = height_fee_pairs
        .into_iter()
        .filter(|(height, _)| *height >= cutoff_block)
        .map(|(_, fee)| fee)
        .collect();

    burn_fees.sort();

    println!(
        "Burn stats for last {} blocks: min={} max={} mean={} avg={}",
        args.blocks,
        burn_fees.first().unwrap(),
        burn_fees.last().unwrap(),
        burn_fees[burn_fees.len() / 2],
        burn_fees.iter().sum::<u64>() / burn_fees.len() as u64
    );
}

// pub fn burns(args: BurnsArgs) {
//     assert!(
//         args.recipients < 10_000,
//         "Cannot query more than 10000 last recipients"
//     );

//     let mut recipients_to_fetch = args.recipients;
//     let mut burn_amounts = vec![];
//     let client = Client::new();

//     while recipients_to_fetch > 0 {
//         let batch_size = recipients_to_fetch.min(250);
//         let res: Value = client
//             .get(format!(
//                 "https://stacks-node-api.{}.stacks.co/extended/v1/burnchain/rewards?limit={}&offset={}",
//                 args.network.to_string().to_ascii_lowercase(), batch_size, args.recipients - recipients_to_fetch
//             ))
//             .send()
//             .unwrap()
//             .json()
//             .unwrap();

//         burn_amounts.extend(res["results"].as_array().unwrap().iter().map(|item| {
//             item["burn_amount"]
//                 .as_str()
//                 .map(|amount| amount.parse::<u64>().unwrap())
//                 .unwrap()
//         }));

//         recipients_to_fetch -= batch_size;
//     }

//     dbg!(&burn_amounts);

//     burn_amounts.sort();

//     println!(
//         "Last {} burnchain winner burns for {}: min={} max={} mean={} avg={}",
//         args.recipients,
//         args.network.to_string().to_ascii_lowercase(),
//         burn_amounts.first().unwrap(),
//         burn_amounts.last().unwrap(),
//         burn_amounts[burn_amounts.len() / 2],
//         burn_amounts.iter().sum::<u64>() / burn_amounts.len() as u64
//     );
// }
