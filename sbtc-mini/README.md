# sbtc-mini

![ci](https://github.com/Trust-Machines/core-eng)

## Introduction

Warning: sbtc-mini is under development and subject to change.

## Clarinet Testing

External dependencies are handled as `requirement` via Clarinet.toml **except** where full control over external contracts prevents fully testing our target contracts.

In this case we pull in the external dependencies into the `tests/contracts` folder and use a script to copy and filter the target contracts into the test folder.

In this way we can reach 100% of the Clarity code under test.

After cloning the repo run the script;

```bash
./scripts/copy-contracts.sh
```

This copies and filters the test contract(s) to the test directory replacing fully qualified external contract calls to
local calls using `.` notation.

The `Clarinet.toml` targets these contracts - see;
