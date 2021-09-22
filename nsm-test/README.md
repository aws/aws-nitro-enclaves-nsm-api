This is a suite of tests for NSM that run in an enclave.
The tests imples that nsm-lib,nsm-driver and nsm-io are built already.
The tests in this directory require an Amazon EC2 instance launched with --enclave-enabled true

# [Prerequisites]

To run the tests using the local Dockerfiles, building the command-executer tool is required.

Enable nitro-cli by:
```
sudo amazon-linux-extras install aws-nitro-enclaves-cli
sudo yum install aws-nitro-enclaves-cli-devel -y
sudo usermod -aG ne ec2-user
```

Log out and log in after that.

# [C++ test](src/main.cc)

This tests libnsm.so.
To build, from the root of the repository run:

```
make .build-nsm-test-cpp-eif
```

To run an enclave with this sample, from the root folder of this repository run:
```
make run-nsm-test-cpp
./command-executer/command_executer_docker_dir/command-executer run --cid 16 --port 5005 --command "/nsm-test/test"
```

# [Rust functional test](src/bin/nsm-check.rs)

This tests the basic functionality of NSM.

## Building
To build, from the root of the repository run:
```
make .build-nsm-check-eif
```
## Running

To run an enclave with this sample, from the root folder of this repository run:
```
make run-nsm-check-eif
./command-executer/command_executer_docker_dir/command-executer run --cid 16 --port 5005 --command "/nsm-check"
```


