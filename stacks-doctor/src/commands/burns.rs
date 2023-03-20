use std::ascii::AsciiExt;

use reqwest::blocking::Client;
use serde_json::Value;

use crate::cli::BurnsArgs;

pub fn burns(args: BurnsArgs) {
    assert!(
        args.recipients < 10_000,
        "Cannot query more than 10000 last recipients"
    );

    let mut recipients_to_fetch = args.recipients;
    let mut burn_amounts = vec![];
    let client = Client::new();

    while recipients_to_fetch > 0 {
        let batch_size = recipients_to_fetch.min(250);
        let res: Value = client
            .get(format!(
                "https://stacks-node-api.{}.stacks.co/extended/v1/burnchain/rewards?limit={}&offset={}",
                args.network.to_string().to_ascii_lowercase(), batch_size, args.recipients - recipients_to_fetch
            ))
            .send()
            .unwrap()
            .json()
            .unwrap();

        burn_amounts.extend(res["results"].as_array().unwrap().iter().map(|item| {
            item["burn_amount"]
                .as_str()
                .map(|amount| amount.parse::<u64>().unwrap())
                .unwrap()
        }));

        recipients_to_fetch -= batch_size;
    }

    burn_amounts.sort();

    println!(
        "Recent {} burnchain winner burns for {}: min={} max={} mean={} avg={}",
        args.recipients,
        args.network.to_string().to_ascii_lowercase(),
        burn_amounts.first().unwrap(),
        burn_amounts.last().unwrap(),
        burn_amounts[burn_amounts.len() / 2],
        burn_amounts.iter().sum::<u64>() / burn_amounts.len() as u64
    );
}
