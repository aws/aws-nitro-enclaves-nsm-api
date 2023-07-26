# Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.

SRC_PATH         = $(dir $(realpath $(lastword $(MAKEFILE_LIST))))
HOST_MACHINE     = $(shell uname -m)
CONTAINER_TAG    = nsm-api
DOCKERFILES_PATH = ${SRC_PATH}/Dockerfiles
BUILD_DOCKERFILE = ${DOCKERFILES_PATH}/Dockerfile.build
TEST_DOCKERFILE  = ${DOCKERFILES_PATH}/Dockerfile.test
COMP_VERSION     = 1.63.0
STABLE           = stable
NIGHTLY          = nightly

.build-${HOST_MACHINE}-${COMP_VERSION}: ${DOCKERFILES_PATH}
	docker image build \
		--build-arg RUST_VERSION=${COMP_VERSION} \
		-t ${CONTAINER_TAG}-${COMP_VERSION} -f ${BUILD_DOCKERFILE} ${SRC_PATH}

.build-${HOST_MACHINE}-${STABLE}:
	docker image build \
		--build-arg RUST_VERSION=${STABLE} \
		-t ${CONTAINER_TAG}-${STABLE} -f ${BUILD_DOCKERFILE} ${SRC_PATH}

.build-${HOST_MACHINE}-${NIGHTLY}: ${DOCKERFILES_PATH}
	docker image build \
		--build-arg RUST_VERSION=${NIGHTLY} \
		-t ${CONTAINER_TAG}-${NIGHTLY} -f ${BUILD_DOCKERFILE} ${SRC_PATH}

nsm-api-${COMP_VERSION}: .build-${HOST_MACHINE}-${COMP_VERSION}
	docker run \
		${CONTAINER_TAG}-${COMP_VERSION} \
		cargo test --all

nsm-api-${STABLE}: .build-${HOST_MACHINE}-${STABLE}
	docker run \
		${CONTAINER_TAG}-${STABLE} \
		/bin/bash -c "cargo build && cargo test --all"

nsm-api-${NIGHTLY}: .build-${HOST_MACHINE}-${NIGHTLY}
	docker run \
		${CONTAINER_TAG}-${NIGHTLY} \
		cargo test --all

rustfmt: nsm-api-${STABLE}
	docker run \
		${CONTAINER_TAG}-${STABLE} \
		cargo fmt -v --all -- --check

clippy: nsm-api-${STABLE}
	docker run \
		${CONTAINER_TAG}-${STABLE} \
		cargo clippy --all

eif_dir:
	mkdir -p eifs/${HOST_MACHINE}/

.build-nsm-test-cpp-docker:
	docker build \
		-f ${TEST_DOCKERFILE} -t nsm-test-cpp --target nsm-test-cpp ${SRC_PATH}

.build-nsm-check-docker:
	docker build \
		-f ${TEST_DOCKERFILE} -t nsm-check --target nsm-check ${SRC_PATH}

.build-nsm-multithread-docker:
	docker build \
		-f ${TEST_DOCKERFILE} -t nsm-multithread --target nsm-multithread ${SRC_PATH}

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
	nitro-cli run-enclave --cpu-count 4 --memory 2048 --eif-path eifs/${HOST_MACHINE}/nsm-multithread.eif --enclave-cid 16 --attach-console
clean:
	cargo clean
