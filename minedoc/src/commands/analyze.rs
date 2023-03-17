use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    thread,
};

use crate::cli::Args;

/*
Run commands below to get a sample log file locally and analyze it:

```
cd stacks-blockchain/testnet/stacks-node;
cargo run -p stacks-node --bin stacks-node -- testnet 2>&1 | tee node.log;

# Set other variables accordingly
minedoc -l /path/to/node.log analyze
```
*/
fn analyze_logs(log_file: PathBuf) -> bool {
    let file = BufReader::new(File::open(log_file).unwrap());
    let mut is_okay = true;

    file.lines().filter_map(Result::ok).for_each(|line| {
        if line.contains("mined anchored block") {
            is_okay = true;
        } else if line.contains("Failure mining") {
            is_okay = false;
            println!("Found problem in logs: {}", line);
        }
    });

    is_okay
}

pub fn analyze(args: Args) {
    let logs_okay = thread::spawn(|| analyze_logs(args.log_file))
        .join()
        .unwrap();

    if logs_okay {
        println!("No problems detected, miner is running well");
    } else {
        println!("Problems detected")
    }
}
