#!/bin/bash
curl --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "importdescriptors", "params": [[{ "desc": "pkh(02e96fe52ef0e22d2f131dd425ce1893073a3c6ad20e8cac36726393dfb4856a4c)", "timestamp": "now", "internal": false, "watchonly": true, "label": "", "keypool": true, "rescan": true }]]}' -H 'content-type: text/plain;' http://abcd:abcd@127.0.0.1:18445/wallet/testdescriptorwallet

# bitcoin-cli importdescriptors '[{ "desc": "<my descriptor>", "timestamp":1455191478, "active": true, "range": [0,100], "label": "<my bech32 wallet>" }]'
