## Nitro Secure Module library

[![version]][crates.io] [![docs]][docs.rs] ![msrv]
[version]: https://img.shields.io/crates/v/aws-nitro-enclaves-nsm-api.svg
[crates.io]: https://crates.io/crates/aws-nitro-enclaves-nsm-api
[docs]: https://img.shields.io/docsrs/aws-nitro-enclaves-nsm-api
[docs.rs]: https://docs.rs/aws-nitro-enclaves-nsm-api
[msrv]: https://img.shields.io/badge/MSRV-1.60.0-blue

This is a collection of helpers which Nitro Enclaves userland
applications can use to communicate with a connected NitroSecureModule (NSM) device.

Various operations can be requested such as:
- PCR query and manipulation
- Attestation
- Entropy

## Prerequisites
An up-to-date RUST toolchain (v1.60.0 or later)

## How To Build
1. Clone the repository
2. Execute `make nsm-api-stable`

## How to Test

# Prerequisites
To run the tests it's required to build the command-executor tool, as follows:
```
make command-executor
```

## License

This project is licensed under the Apache-2.0 License.

## Security issue notifications

If you discover a potential security issue in the Nitro Enclaves NSM API, we ask that you notify AWS
Security via our
[vulnerability reporting page](https://aws.amazon.com/security/vulnerability-reporting/).
Please do **not** create a public GitHub issue.
