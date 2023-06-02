#!/bin/bash
curl --data-binary '{"jsonrpc": "1.0", "id": "curltest", "method": "importdescriptor", "params": [{ "desc": "...", "timestamp": "now", "internal": false, "watchonly": true, "label": "", "keypool": true, "rescan": true }]}' -H 'content-type: text/plain;' http://abcd:abcd@127.0.0.1:18445/

# bitcoin-cli importdescriptors '[{ "desc": "<my descriptor>", "timestamp":1455191478, "active": true, "range": [0,100], "label": "<my bech32 wallet>" }]'
