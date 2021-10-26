## Nitro Secure Module library

This is a collection of helpers which Nitro Enclaves userland
applications can use to communicate with a connected NitroSecureModule (NSM) device.

Various operations can be requested such as:
- PCR query and manipulation
- Attestation
- Entropy

## Prerequisites
An up-to-date RUST toolchain (v1.41.0 or later)

## How To Build
1. Clone the repository
2. Execute `cargo build`

## How to integrate this module in your Nitro Enclaves project
TODO: Link to AWS documentation

## License

This project is licensed under the Apache-2.0 License.

## Security issue notifications

If you discover a potential security issue in the Nitro Enclaves NSM API, we ask that you notify AWS
Security via our
[vulnerability reporting page](https://aws.amazon.com/security/vulnerability-reporting/).
Please do **not** create a public GitHub issue.
