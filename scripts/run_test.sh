#!/bin/bash

docker system prune --all --force
make run-nsm-test-cpp
./command-executer/command_executer_docker_dir/command-executer run --cid 16 --port 5005 --command "/nsm-test/test" --no-wait

RESULT=$?
if [ $RESULT -eq 0]; then
	echo "The test-cpp test has PASSED"
else
	echo "The test-cpp test has FAILED"
fi

nitro-cli terminate-enclave --all

make run-nsm-check-eif
./command-executer/command_executer_docker_dir/command-executer run --cid 16 --port 5005 --command "/nsm-check" --no-wait

RESULT=$?
if [ $RESULT -eq 0]; then
	echo "The nsm-check test has PASSED"
else
	echo "The nsm-check test has FAILED"
fi

nitro-cli terminate-enclave --all
