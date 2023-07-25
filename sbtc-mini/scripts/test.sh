#!/bin/sh
mkdir -p .test
mkdir -p .test-2
mkdir -p .test-3
mkdir -p .coverage
#clarinet run --allow-write --allow-read ext/generate-tests.ts
#clarinet run --allow-write --allow-read ext/generate-tests-2.ts
clarinet run --allow-write --allow-read ext/generate-tests-3.ts
#clarinet test --coverage .coverage/lcov.info .test
#clarinet test --coverage .coverage/lcov.info .test-2
clarinet test --coverage .coverage/lcov.info .test-3
