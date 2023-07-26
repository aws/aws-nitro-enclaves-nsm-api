# 0.4.0

Changes:
* Update minimum supported rust version to 1.63.0.

# 0.3.0

Changes:
* Introduce default enabled nix feature flag to allow for non nix usage (#41)
* Update minimum supported Rust version to v1.60

Fixes:
* minor clippy fixes
* update references to IETF documents
* Fix typos
* Update Dockerfile.build (#31)
* Fix and simplify Dockerfiles

Updates:
* update nix to 0.26 and fix deprecation warnings (#46)
* update signal-hook requirement from =0.1.8 to =0.3.15
* update cbindgen requirement from 0.21 to 0.24
* update vsock requirement from 0.2 to 0.3

# 0.2.1

Fixes:
* build container on other distros than Amazon Linux 2
* build container permissions
* nsm-lib header generation

Updates:
* default build container rust version to 1.58.1
* cbindgen to 0.21

# 0.2.0

* Added top level crate
* Reorganize nsm-driver and nsm-io into top level crate
* Changed authors in Cargo.toml
