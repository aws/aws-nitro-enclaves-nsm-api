# Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.

SRC_PATH         = $(dir $(realpath $(lastword $(MAKEFILE_LIST))))
HOST_MACHINE     = $(shell uname -m)
CONTAINER_TAG    = nsm-api
DOCKERFILES_PATH = ${SRC_PATH}/Dockerfiles
BUILD_DOCKERFILE = ${DOCKERFILES_PATH}/Dockerfile.build
TEST_DOCKERFILE  = ${DOCKERFILES_PATH}/Dockerfile.test
COMP_VERSION     = $(shell grep "rust-version" Cargo.toml | head -n 1 | cut -d' ' -f3 | tr -d '"')
STABLE           = stable
NIGHTLY          = nightly
VERSION = $(shell grep "version" Cargo.toml | head -n 1 | cut -d' ' -f3 | tr -d '"')
RELEASE_DIR	= target/release
RUSTC_STATIC_LIBS = $(shell cargo rustc -q -- --print=native-static-libs)

LIBNSM_ABI_VERSION = 0
LIBNSM_SONAME = libnsm.so
LIBNSM_SONAME_ABI_VERSION = $(LIBNSM_SONAME).$(LIBNSM_ABI_VERSION)
LIBNSM_SONAME_VERSION = $(LIBNSM_SONAME).$(VERSION)

ifeq ($(PREFIX),)
	PREFIX := /usr/local
endif

LIBDIR ?= $(PREFIX)/lib
INCLUDEDIR ?= $(PREFIX)/include

.PHONY: libnsm.pc nsm-lib install clean

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

nsm-lib:
	cargo build --package $@ --release

libnsm.pc: libnsm.pc.in
	rm -f $@ $@-t
	sed -e 's|@prefix@|$(PREFIX)|' \
	    -e 's|@libdir@|$(LIBDIR)|' \
	    -e 's|@includedir@|$(INCLUDEDIR)|' \
	    -e 's|@PACKAGE_NAME@|libnsm|' \
	    -e 's|@PACKAGE_VERSION@|$(VERSION)|' \
	    -e 's|@LIBS_PRIVATE@|$(RUSTC_STATIC_LIBS)|' \
	    libnsm.pc.in > $@-t
	mv $@-t $@

install: nsm-lib libnsm.pc
	install -d $(DESTDIR)$(LIBDIR)
	install -d $(DESTDIR)$(LIBDIR)/pkgconfig
	install -d $(DESTDIR)$(INCLUDEDIR)
	install -m 644 $(RELEASE_DIR)/nsm.h $(DESTDIR)$(INCLUDEDIR)
	install -m 644 libnsm.pc $(DESTDIR)$(LIBDIR)/pkgconfig
	patchelf --set-soname $(LIBNSM_SONAME_ABI_VERSION) $(RELEASE_DIR)/$(LIBNSM_SONAME)
	install -m 755 $(RELEASE_DIR)/$(LIBNSM_SONAME) $(DESTDIR)$(LIBDIR)/$(LIBNSM_SONAME_VERSION)
	install -m 644 $(RELEASE_DIR)/libnsm.a $(DESTDIR)$(LIBDIR)
	cd $(DESTDIR)$(LIBDIR)/ ; ln -sf $(LIBNSM_SONAME_VERSION) $(LIBNSM_SONAME_ABI_VERSION) ; ln -sf $(LIBNSM_SONAME_ABI_VERSION) $(LIBNSM_SONAME)
clean:
	cargo clean
