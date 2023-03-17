use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

/*
Run commands below to get a sample log file locally and analyze it:

```
cd stacks-blockchain/testnet/stacks-node;
cargo run -p stacks-node --bin stacks-node -- testnet 2>&1 | tee node.log;

# Set other variables accordingly
minedoc -l /path/to/node.log analyze
```
*/
pub fn analyze_logs(log_file: PathBuf) -> bool {
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
