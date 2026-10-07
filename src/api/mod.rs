// Copyright 2020-2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]
#![allow(clippy::upper_case_acronyms)]
//! NitroSecurityModule IO
//! # Overview
//! This module contains the structure definitions that allows data interchange between
//! a NitroSecureModule and the client using it. It uses CBOR to encode the data to allow
//! easy IPC between components.

// BTreeMap preserves ordering, which makes the tests easier to write
use std::collections::{BTreeMap, BTreeSet};
use std::io::Error as IoError;
use std::result;

use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

pub mod cbor;
use cbor::{from_slice, to_vec, Error as CborError};

#[derive(Debug)]
/// Possible error types return from this library.
pub enum Error {
    /// An IO error of type `std::io::Error`
    Io(IoError),
    /// A CBOR encoding or decoding error.
    Cbor(CborError),
}

/// Result type return nsm-io::Error on failure.
pub type Result<T> = result::Result<T, Error>;

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl From<CborError> for Error {
    fn from(error: CborError) -> Self {
        Error::Cbor(error)
    }
}

/// List of error codes that the NSM module can return as part of a Response
#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorCode {
    /// No errors
    Success,

    /// Input argument(s) invalid
    InvalidArgument,

    /// PlatformConfigurationRegister index out of bounds
    InvalidIndex,

    /// The received response does not correspond to the earlier request
    InvalidResponse,

    /// PlatformConfigurationRegister is in read-only mode and the operation
    /// attempted to modify it
    ReadOnlyIndex,

    /// Given request cannot be fulfilled due to missing capabilities
    InvalidOperation,

    /// Operation succeeded but provided output buffer is too small
    BufferTooSmall,

    /// The user-provided input is too large
    InputTooLarge,

    /// NitroSecureModule cannot fulfill request due to internal errors
    InternalError,
}

/// Operations that a NitroSecureModule should implement. Assumes 64K registers will be enough for everyone.
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Request {
    /// Read data from PlatformConfigurationRegister at `index`
    DescribePCR {
        /// index of the PCR to describe
        index: u16,
    },

    /// Extend PlatformConfigurationRegister at `index` with `data`
    ExtendPCR {
        /// index the PCR to extend
        index: u16,

        #[serde(with = "serde_bytes")]
        /// data to extend it with
        data: Vec<u8>,
    },

    /// Lock PlatformConfigurationRegister at `index` from further modifications
    LockPCR {
        /// index to lock
        index: u16,
    },

    /// Lock PlatformConfigurationRegisters at indexes `[0, range)` from further modifications
    LockPCRs {
        /// number of PCRs to lock, starting from index 0
        range: u16,
    },

    /// Return capabilities and version of the connected NitroSecureModule. Clients are recommended to decode
    /// major_version and minor_version first, and use an appropriate structure to hold this data, or fail
    /// if the version is not supported.
    DescribeNSM,

    /// Requests the NSM to create an AttestationDoc and sign it with it's private key to ensure
    /// authenticity.
    Attestation {
        /// Includes additional user data in the AttestationDoc.
        user_data: Option<ByteBuf>,

        /// Includes an additional nonce in the AttestationDoc.
        nonce: Option<ByteBuf>,

        /// Includes a user provided public key in the AttestationDoc.
        public_key: Option<ByteBuf>,
    },

    /// Requests entropy from the NSM side.
    GetRandom,
}

/// Responses received from a NitroSecureModule as a result of a Request
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Response {
    /// returns the current PlatformConfigurationRegister state
    DescribePCR {
        /// true if the PCR is read-only, false otherwise
        lock: bool,
        #[serde(with = "serde_bytes")]
        /// the current value of the PCR
        data: Vec<u8>,
    },

    /// returned if PlatformConfigurationRegister has been successfully extended
    ExtendPCR {
        #[serde(with = "serde_bytes")]
        /// The new value of the PCR after extending the data into the register.
        data: Vec<u8>,
    },

    /// returned if PlatformConfigurationRegister has been successfully locked
    LockPCR,

    /// returned if PlatformConfigurationRegisters have been successfully locked
    LockPCRs,

    /// returns the runtime configuration of the NitroSecureModule
    DescribeNSM {
        /// Breaking API changes are denoted by `major_version`
        version_major: u16,
        /// Minor API changes are denoted by `minor_version`. Minor versions should be backwards compatible.
        version_minor: u16,
        /// Patch version. These are security and stability updates and do not affect API.
        version_patch: u16,
        /// `module_id` is an identifier for a singular NitroSecureModule
        module_id: String,
        /// The maximum number of PCRs exposed by the NitroSecureModule.
        max_pcrs: u16,
        /// The PCRs that are read-only.
        locked_pcrs: BTreeSet<u16>,
        /// The digest of the PCR Bank
        digest: Digest,
    },

    /// A response to an Attestation Request containing the CBOR-encoded AttestationDoc and the
    /// signature generated from the doc by the NitroSecureModule
    Attestation {
        /// A signed COSE structure containing a CBOR-encoded AttestationDocument as the payload.
        #[serde(with = "serde_bytes")]
        document: Vec<u8>,
    },

    /// A response containing a number of bytes of entropy.
    GetRandom {
        #[serde(with = "serde_bytes")]
        /// The random bytes.
        random: Vec<u8>,
    },

    /// An error has occured, and the NitroSecureModule could not successfully complete the operation
    Error(ErrorCode),
}

/// The digest implementation used by a NitroSecureModule
#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum Digest {
    /// SHA256
    SHA256,
    /// SHA384
    SHA384,
    /// SHA512
    SHA512,
}

/// An attestation response.  This is also used for sealing
/// data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AttestationDoc {
    /// Issuing NSM ID
    pub module_id: String,

    /// The digest function used for calculating the register values
    /// Can be: "SHA256" | "SHA512"
    pub digest: Digest,

    /// UTC time when document was created expressed as milliseconds since Unix Epoch
    pub timestamp: u64,

    /// Map of all locked PCRs at the moment the attestation document was generated
    pub pcrs: BTreeMap<usize, ByteBuf>,

    /// The infrastucture certificate used to sign the document, DER encoded
    pub certificate: ByteBuf,
    /// Issuing CA bundle for infrastructure certificate
    pub cabundle: Vec<ByteBuf>,

    /// An optional DER-encoded key the attestation consumer can use to encrypt data with
    pub public_key: Option<ByteBuf>,

    /// Additional signed user data, as defined by protocol.
    pub user_data: Option<ByteBuf>,

    /// An optional cryptographic nonce provided by the attestation consumer as a proof of
    /// authenticity.
    pub nonce: Option<ByteBuf>,
}

impl AttestationDoc {
    /// Creates a new AttestationDoc.
    ///
    /// # Arguments
    ///
    /// * module_id: a String representing the name of the NitroSecureModule
    /// * digest: nsm_io::Digest that describes what the PlatformConfigurationRegisters
    ///   contain
    /// * pcrs: BTreeMap containing the index to PCR value
    /// * certificate: the serialized certificate that will be used to sign this AttestationDoc
    /// * cabundle: the serialized set of certificates up to the root of trust certificate that
    ///   emitted `certificate`
    /// * user_data: optional user definted data included in the AttestationDoc
    /// * nonce: optional cryptographic nonce that will be included in the AttestationDoc
    /// * public_key: optional DER-encoded public key that will be included in the AttestationDoc
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        module_id: String,
        digest: Digest,
        timestamp: u64,
        pcrs: BTreeMap<usize, Vec<u8>>,
        certificate: Vec<u8>,
        cabundle: Vec<Vec<u8>>,
        user_data: Option<Vec<u8>>,
        nonce: Option<Vec<u8>>,
        public_key: Option<Vec<u8>>,
    ) -> Self {
        let mut pcrs_serialized = BTreeMap::new();

        for (i, pcr) in pcrs.into_iter() {
            let pcr = ByteBuf::from(pcr);
            pcrs_serialized.insert(i, pcr);
        }

        let cabundle_serialized = cabundle.into_iter().map(ByteBuf::from).collect();

        AttestationDoc {
            module_id,
            digest,
            timestamp,
            pcrs: pcrs_serialized,
            cabundle: cabundle_serialized,
            certificate: ByteBuf::from(certificate),
            user_data: user_data.map(ByteBuf::from),
            nonce: nonce.map(ByteBuf::from),
            public_key: public_key.map(ByteBuf::from),
        }
    }

    /// Helper function that converts an AttestationDoc structure to its CBOR representation
    pub fn to_binary(&self) -> Vec<u8> {
        // This should not fail
        to_vec(self).unwrap()
    }

    /// Helper function that parses a CBOR representation of an AttestationDoc and creates the
    /// structure from it, if possible.
    pub fn from_binary(bin: &[u8]) -> Result<Self> {
        from_slice(bin).map_err(Error::Cbor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attestationdoc_binary_encode() {
        let mut pcrs = BTreeMap::new();
        pcrs.insert(1, vec![1, 2, 3]);
        pcrs.insert(2, vec![4, 5, 6]);
        pcrs.insert(3, vec![7, 8, 9]);

        let doc1 = AttestationDoc::new(
            "abcd".to_string(),
            Digest::SHA256,
            1234,
            pcrs,
            vec![42; 10],
            vec![],
            Some(vec![255; 10]),
            None,
            None,
        );
        let bin1 = doc1.to_binary();
        let doc2 = AttestationDoc::from_binary(&bin1).unwrap();
        let bin2 = doc2.to_binary();
        assert_eq!(doc1, doc2);
        assert_eq!(bin1, bin2);
    }

    fn unhex(s: &str) -> Vec<u8> {
        hex::decode(s).unwrap()
    }

    // Expected bytes were produced by serde_cbor 0.11.2 on the 0.5.2 tree.
    // They pin the wire format across the ciborium migration.

    #[test]
    fn request_encoding_matches_serde_cbor() {
        let cases: Vec<(Request, &str)> = vec![
            (Request::DescribePCR { index: 3 }, "a16b4465736372696265504352a165696e64657803"),
            (
                Request::ExtendPCR {
                    index: 4,
                    data: vec![1, 2, 3],
                },
                "a169457874656e64504352a265696e64657804646461746143010203",
            ),
            (Request::LockPCR { index: 5 }, "a1674c6f636b504352a165696e64657805"),
            (Request::LockPCRs { range: 16 }, "a1684c6f636b50435273a16572616e676510"),
            (Request::DescribeNSM, "6b44657363726962654e534d"),
            (
                Request::Attestation {
                    user_data: Some(ByteBuf::from(vec![9, 9])),
                    nonce: None,
                    public_key: Some(ByteBuf::from(vec![7])),
                },
                "a16b4174746573746174696f6ea369757365725f64617461420909656e6f6e6365f66a7075626c69635f6b65794107",
            ),
            (Request::GetRandom, "6947657452616e646f6d"),
        ];
        for (req, expected) in cases {
            let bytes = to_vec(&req).unwrap();
            assert_eq!(bytes, unhex(expected), "{req:?}");
            let back: Request = from_slice(&bytes).unwrap();
            assert_eq!(to_vec(&back).unwrap(), bytes, "{req:?} round trip");
        }
    }

    #[test]
    fn response_encoding_matches_serde_cbor() {
        let cases: Vec<(Response, &str)> = vec![
            (
                Response::DescribePCR {
                    lock: true,
                    data: vec![0xaa, 0xbb],
                },
                "a16b4465736372696265504352a2646c6f636bf5646461746142aabb",
            ),
            (Response::ExtendPCR { data: vec![1] }, "a169457874656e64504352a164646174614101"),
            (Response::LockPCR, "674c6f636b504352"),
            (Response::LockPCRs, "684c6f636b50435273"),
            (
                Response::DescribeNSM {
                    version_major: 1,
                    version_minor: 2,
                    version_patch: 3,
                    module_id: "i-abc".into(),
                    max_pcrs: 32,
                    locked_pcrs: vec![0u16, 1].into_iter().collect(),
                    digest: Digest::SHA384,
                },
                "a16b44657363726962654e534da76d76657273696f6e5f6d616a6f72016d76657273696f6e5f6d696e6f72026d76657273696f6e5f706174636803696d6f64756c655f696465692d616263686d61785f7063727318206b6c6f636b65645f706372738200016664696765737466534841333834",
            ),
            (
                Response::Attestation {
                    document: vec![0xd2, 0x84],
                },
                "a16b4174746573746174696f6ea168646f63756d656e7442d284",
            ),
            (
                Response::GetRandom {
                    random: vec![4, 4, 4],
                },
                "a16947657452616e646f6da16672616e646f6d43040404",
            ),
            (
                Response::Error(ErrorCode::InvalidIndex),
                "a1654572726f726c496e76616c6964496e646578",
            ),
        ];
        for (resp, expected) in cases {
            let bytes = to_vec(&resp).unwrap();
            assert_eq!(bytes, unhex(expected), "{resp:?}");
            let back: Response = from_slice(&bytes).unwrap();
            assert_eq!(to_vec(&back).unwrap(), bytes, "{resp:?} round trip");
        }
    }

    #[test]
    fn attestation_doc_encoding_matches_serde_cbor() {
        let mut pcrs = BTreeMap::new();
        pcrs.insert(1, vec![1, 2, 3]);
        pcrs.insert(2, vec![4, 5, 6]);
        let doc = AttestationDoc::new(
            "abcd".to_string(),
            Digest::SHA256,
            1234,
            pcrs,
            vec![42; 10],
            vec![],
            Some(vec![255; 10]),
            None,
            None,
        );
        assert_eq!(doc.to_binary(), unhex("a9696d6f64756c655f6964646162636466646967657374665348413235366974696d657374616d701904d26470637273a2014301020302430405066b63657274696669636174654a2a2a2a2a2a2a2a2a2a2a68636162756e646c65806a7075626c69635f6b6579f669757365725f646174614affffffffffffffffffff656e6f6e6365f6"));
    }

    #[test]
    fn from_binary_rejects_trailing_bytes() {
        let mut bytes = to_vec(&Request::DescribeNSM).unwrap();
        bytes.push(0x00);
        assert!(from_slice::<Request>(&bytes).is_err());
    }
}
