# Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.

HOST_MACHINE    = $(shell uname -m)
CONTAINER_TAG   = nsm-api
DOCKERFILE_PATH = Dockerfiles/Dockerfile.build
COMP_VERSION    = 1.40
STABLE          = stable
NIGHTLY         = nightly

.build-${HOST_MACHINE}-${COMP_VERSION}: ${DOCKERFILE_PATH}
	docker image build \
		--build-arg HOST_MACHINE=${HOST_MACHINE} \
		--build-arg RUST_VERSION=${COMP_VERSION} \
		-t ${CONTAINER_TAG}-${COMP_VERSION} -f ${DOCKERFILE_PATH} .

.build-${HOST_MACHINE}-${STABLE}:
	docker image build \
		--build-arg HOST_MACHINE=${HOST_MACHINE} \
		--build-arg RUST_VERSION=${STABLE} \
		-t ${CONTAINER_TAG}-${STABLE} -f ${DOCKERFILE_PATH} .

.build-${HOST_MACHINE}-${NIGHTLY}: ${DOCKERFILE_PATH}
	docker image build \
		--build-arg HOST_MACHINE=${HOST_MACHINE} \
		--build-arg RUST_VERSION=${NIGHTLY} \
		-t ${CONTAINER_TAG}-${NIGHTLY} -f ${DOCKERFILE_PATH} .

nsm-api-${COMP_VERSION}: .build-${HOST_MACHINE}-${COMP_VERSION}
	docker run \
		${CONTAINER_TAG}-${COMP_VERSION} \
		cargo test --all

nsm-api-${STABLE}: .build-${HOST_MACHINE}-${STABLE}
	docker run \
		-v /home/ec2-user/aws-nitro-enclaves-nsm-api/:/build \
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

clean:
	rm -rf ./target
