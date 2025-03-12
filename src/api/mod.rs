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
use serde_cbor::error::Error as CborError;
use serde_cbor::{from_slice, to_vec};

#[derive(Debug)]
/// Possible error types return from this library.
pub enum Error {
    /// An IO error of type `std::io::Error`
    Io(IoError),
    /// A CBOR ser/de error of type `serde_cbor::error::Error`.
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
}
