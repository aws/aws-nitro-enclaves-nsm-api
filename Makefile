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

clean:
	rm -rf ./target
