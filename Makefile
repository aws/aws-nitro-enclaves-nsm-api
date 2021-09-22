# Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.

HOST_MACHINE    = $(shell uname -m)
CONTAINER_TAG   = nsm-api
DOCKERFILE_PATH = Dockerfiles/Dockerfile

.build-${HOST_MACHINE}-1.41: ${DOCKERFILE_PATH}.1.41
	docker image build --build-arg HOST_MACHINE=${HOST_MACHINE} -t ${CONTAINER_TAG}-1.41 -f ${DOCKERFILE_PATH}.1.41 .

.build-${HOST_MACHINE}-stable: ${DOCKERFILE_PATH}.stable
	docker image build --build-arg HOST_MACHINE=${HOST_MACHINE} -t ${CONTAINER_TAG}-stable -f ${DOCKERFILE_PATH}.stable .

.build-${HOST_MACHINE}-nightly: ${DOCKERFILE_PATH}.nightly
	docker image build --build-arg HOST_MACHINE=${HOST_MACHINE} -t ${CONTAINER_TAG}-nightly -f ${DOCKERFILE_PATH}.nightly .

nsm-api-1.41: .build-${HOST_MACHINE}-1.41
	docker run \
		${CONTAINER_TAG}-1.41 \
		cargo test --all

nsm-api-stable: .build-${HOST_MACHINE}-stable
	docker run \
		-v /home/ec2-user/aws-nitro-enclaves-nsm-api/:/build \
		${CONTAINER_TAG}-stable \
		/bin/bash -c "cargo build && cargo test --all"

nsm-api-nightly: .build-${HOST_MACHINE}-nightly
	docker run \
		${CONTAINER_TAG}-nightly \
		cargo test --all

rustfmt: nsm-api-stable
	docker run \
		${CONTAINER_TAG}-stable \
		cargo fmt -v --all -- --check

clippy: nsm-api-stable
	docker run \
		${CONTAINER_TAG}-stable \
		cargo clippy --all

eif_dir:
	mkdir -p eifs/${HOST_MACHINE}/

command-executer: command-executer
	git clone https://github.com/aws/aws-nitro-enclaves-cli
	cd aws-nitro-enclaves-cli && make command-executer
	cp -r aws-nitro-enclaves-cli/build/command-executer .
	rm -rf aws-nitro-enclaves-cli

.build-nsm-test-cpp-docker: command-executer
	docker build -f Dockerfiles/Dockerfile.test -t nsm-test-cpp --target nsm-test-cpp .

.build-nsm-check-docker: command-executer
	docker build -f Dockerfiles/Dockerfile.test -t nsm-check --target nsm-check .

.build-nsm-multithread-docker: command-executer
	docker build -f Dockerfiles/Dockerfile.test -t nsm-multithread --target nsm-multithread .

.build-nsm-test-cpp-eif: .build-nsm-test-cpp-docker eif_dir
	nitro-cli build-enclave --docker-uri nsm-test-cpp:latest --output-file eifs/${HOST_MACHINE}/nsm-test-cpp.eif

.build-nsm-check-eif: .build-nsm-check-docker eif_dir
	nitro-cli build-enclave --docker-uri nsm-check:latest --output-file eifs/${HOST_MACHINE}/nsm-check.eif

.build-nsm-multithread-eif: .build-nsm-multithread-docker eif_dir
	nitro-cli build-enclave --docker-uri nsm-multithread:latest --output-file eifs/${HOST_MACHINE}/nsm-multithread.eif

run-nsm-test-cpp: .build-nsm-test-cpp-eif
	nitro-cli run-enclave --cpu-count 4 --memory 2048 --eif-path eifs/${HOST_MACHINE}/nsm-test-cpp.eif --enclave-cid 16

run-nsm-check-eif: .build-nsm-check-eif
	nitro-cli run-enclave --cpu-count 4 --memory 2048 --eif-path eifs/${HOST_MACHINE}/nsm-check.eif --enclave-cid 16

run-nsm-multithread-eif: .build-nsm-multithread-eif
	nitro-cli run-enclave --cpu-count 4 --memory 2048 --eif-path eifs/${HOST_MACHINE}/nsm-multithread.eif --enclave-cid 16 --debug-mode

clean:
	rm -rf ./target
