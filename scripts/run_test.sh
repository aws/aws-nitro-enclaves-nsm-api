#!/bin/sh

NUM_TESTS=2
tests_passed=0

set -eu
if [ $# -eq 1 ]; then
	if [ "$1" = "CLI_CHECK" ]; then
		docker system prune --all --force
	fi
fi

make run-nsm-test-cpp
./command-executer/command_executer_docker_dir/command-executer run --cid 16 --port 5005 --command "/nsm-test/test"

RESULT=$?
if [ $RESULT -eq 0 ]; then
	echo "The test-cpp test has PASSED"
	tests_passed=$((tests_passed+1))
else
	echo "The test-cpp test has FAILED"
fi

nitro-cli terminate-enclave --all

make run-nsm-check-eif
./command-executer/command_executer_docker_dir/command-executer run --cid 16 --port 5005 --command "/nsm-check"

RESULT=$?
if [ $RESULT -eq 0 ]; then
	echo "The nsm-check test has PASSED"
	tests_passed=$((tests_passed+1))
else
	echo "The nsm-check test has FAILED"
fi

nitro-cli terminate-enclave --all

echo "$tests_passed/$NUM_TESTS tests passed"
