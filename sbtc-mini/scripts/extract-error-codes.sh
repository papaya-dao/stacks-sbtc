#!/bin/sh
mkdir -p .test
mkdir -p .coverage
clarinet run --allow-write --allow-read ext/extract-error-codes.ts

