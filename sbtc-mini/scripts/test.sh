#!/bin/sh
mkdir -p .test
mkdir -p .test-2
mkdir -p .coverage
clarinet run --allow-write ext/generate-tests.ts
clarinet run --allow-write ext/generate-tests-2.ts
clarinet test --coverage .coverage/lcov.info .test
clarinet test --coverage .coverage/lcov.info .test-2
