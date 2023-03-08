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
use minicbor::{Decode, Encode};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Error as IoError;
use std::result;

#[derive(Debug)]
/// Possible error types return from this library.
pub enum Error {
    /// An IO error of type `std::io::Error`
    Io(IoError),
    /// An error attempting to decode with the `minicbor` library.
    CborDecode(minicbor::decode::Error),
}

/// Result type return nsm-io::Error on failure.
pub type Result<T> = result::Result<T, Error>;

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl From<minicbor::decode::Error> for Error {
    fn from(error: minicbor::decode::Error) -> Self {
        Error::CborDecode(error)
    }
}

/// List of error codes that the NSM module can return as part of a Response
#[repr(C)]
#[derive(Debug, Encode, Decode)]
pub enum ErrorCode {
    /// No errors
     #[n(0)] Success,

    /// Input argument(s) invalid
     #[n(1)] InvalidArgument,

    /// PlatformConfigurationRegister index out of bounds
     #[n(2)] InvalidIndex,

    /// The received response does not correspond to the earlier request
     #[n(3)] InvalidResponse,

    /// PlatformConfigurationRegister is in read-only mode and the operation
    /// attempted to modify it
     #[n(4)] ReadOnlyIndex,

    /// Given request cannot be fulfilled due to missing capabilities
     #[n(5)] InvalidOperation,

    /// Operation succeeded but provided output buffer is too small
     #[n(6)] BufferTooSmall,

    /// The user-provided input is too large
     #[n(7)] InputTooLarge,

    /// NitroSecureModule cannot fulfill request due to internal errors
     #[n(8)] InternalError,
}

/// Operations that a NitroSecureModule should implement. Assumes 64K registers will be enough for everyone.
#[derive(Debug, Encode, Decode)]
#[non_exhaustive]
pub enum Request {
    /// Read data from PlatformConfigurationRegister at `index`
     #[n(0)] DescribePCR {
        /// index of the PCR to describe
        #[n(0)] index: u16,
    },

    /// Extend PlatformConfigurationRegister at `index` with `data`
    #[n(1)] ExtendPCR {
        /// index the PCR to extend
        #[n(0)] index: u16,

        /// data to extend it with
        #[n(1)] data: Vec<u8>,
    },

    /// Lock PlatformConfigurationRegister at `index` from further modifications
    #[n(2)] LockPCR {
        /// index to lock
        #[n(0)] index: u16,
    },

    /// Lock PlatformConfigurationRegisters at indexes `[0, range)` from further modifications
    #[n(3)] LockPCRs {
        /// number of PCRs to lock, starting from index 0
        #[n(0)] range: u16,
    },

    /// Return capabilities and version of the connected NitroSecureModule. Clients are recommended to decode
    /// major_version and minor_version first, and use an appropriate structure to hold this data, or fail
    /// if the version is not supported.
    #[n(4)] DescribeNSM,

    /// Requests the NSM to create an AttestationDoc and sign it with it's private key to ensure
    /// authenticity.
    #[n(5)] Attestation {
        /// Includes additional user data in the AttestationDoc.
        #[n(0)] user_data: Option<Vec<u8>>,

        /// Includes an additional nonce in the AttestationDoc.
        #[n(1)] nonce: Option<Vec<u8>>,

        /// Includes a user provided public key in the AttestationDoc.
        #[n(2)] public_key: Option<Vec<u8>>,
    },

    /// Requests entropy from the NSM side.
    #[n(6)] GetRandom,
}

/// Responses received from a NitroSecureModule as a result of a Request
#[derive(Debug, Encode, Decode)]
#[non_exhaustive]
pub enum Response {
    /// returns the current PlatformConfigurationRegister state
    #[n(0)]  DescribePCR {
        /// true if the PCR is read-only, false otherwise
        #[n(0)] lock: bool,
        /// the current value of the PCR
        #[n(1)] data: Vec<u8>,
    },

    /// returned if PlatformConfigurationRegister has been successfully extended
    #[n(1)] ExtendPCR {
        /// The new value of the PCR after extending the data into the register.
        #[n(0)] data: Vec<u8>,
    },

    /// returned if PlatformConfigurationRegister has been successfully locked
    #[n(2)] LockPCR,

    /// returned if PlatformConfigurationRegisters have been successfully locked
    #[n(3)] LockPCRs,

    /// returns the runtime configuration of the NitroSecureModule
    #[n(4)] DescribeNSM {
        /// Breaking API changes are denoted by `major_version`
        #[n(0)] version_major: u16,
        /// Minor API changes are denoted by `minor_version`. Minor versions should be backwards compatible.
        #[n(1)] version_minor: u16,
        /// Patch version. These are security and stability updates and do not affect API.
        #[n(2)] version_patch: u16,
        /// `module_id` is an identifier for a singular NitroSecureModule
        #[n(3)] module_id: String,
        /// The maximum number of PCRs exposed by the NitroSecureModule.
        #[n(4)] max_pcrs: u16,
        /// The PCRs that are read-only.
        #[n(5)] locked_pcrs: BTreeSet<u16>,
        /// The digest of the PCR Bank
        #[n(6)] digest: Digest,
    },

    /// A response to an Attestation Request containing the CBOR-encoded AttestationDoc and the
    /// signature generated from the doc by the NitroSecureModule
    #[n(5)] Attestation {
        /// A signed COSE structure containing a CBOR-encoded AttestationDocument as the payload.
       #[n(0)] document: Vec<u8>,
    },

    /// A response containing a number of bytes of entropy.
    #[n(6)] GetRandom {
        /// The random bytes.
        #[n(0)] random: Vec<u8>,
    },

    /// An error has occured, and the NitroSecureModule could not successfully complete the operation
    #[n(7)] Error(#[n(0)] ErrorCode),
}

/// The digest implementation used by a NitroSecureModule
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Encode, Decode)]
pub enum Digest {
    /// SHA256
    #[n(0)] SHA256,
    /// SHA384
    #[n(1)] SHA384,
    /// SHA512
    #[n(2)] SHA512,
}

/// An attestation response.  This is also used for sealing
/// data.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct AttestationDoc {
    /// Issuing NSM ID
    #[n(0)] pub module_id: String,

    /// The digest function used for calculating the register values
    /// Can be: "SHA256" | "SHA512"
    #[n(1)] pub digest: Digest,

    /// UTC time when document was created expressed as milliseconds since Unix Epoch
    #[n(2)] pub timestamp: u64,

    /// Map of all locked PCRs at the moment the attestation document was generated
    #[n(3)] pub pcrs: BTreeMap<usize, Vec<u8>>,

    /// The infrastucture certificate used to sign the document, DER encoded
    #[n(4)] pub certificate: Vec<u8>,

    /// Issuing CA bundle for infrastructure certificate
    #[n(5)] pub cabundle: Vec<Vec<u8>>,

    /// An optional DER-encoded key the attestation consumer can use to encrypt data with
    #[n(6)] pub public_key: Option<Vec<u8>>,

    /// Additional signed user data, as defined by protocol.
    #[n(7)] pub user_data: Option<Vec<u8>>,

    /// An optional cryptographic nonce provided by the attestation consumer as a proof of
    /// authenticity.
    #[n(8)] pub nonce: Option<Vec<u8>>,
}

impl AttestationDoc {
    /// Creates a new AttestationDoc.
    ///
    /// # Arguments
    ///
    /// * module_id: a String representing the name of the NitroSecureModule
    /// * digest: nsm_io::Digest that describes what the PlatformConfigurationRegisters
    ///           contain
    /// * pcrs: BTreeMap containing the index to PCR value
    /// * certificate: the serialized certificate that will be used to sign this AttestationDoc
    /// * cabundle: the serialized set of certificates up to the root of trust certificate that
    ///             emitted `certificate`
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

        AttestationDoc {
            module_id,
            digest,
            timestamp,
            pcrs,
            cabundle,
            certificate,
            user_data,
            nonce,
            public_key,
        }
    }

    /// Helper function that converts an AttestationDoc structure to its CBOR representation
    pub fn to_binary(&self) -> Vec<u8> {
        // `to_vec` is infallible: https://gitlab.com/twittner/minicbor/-/blob/develop/minicbor/src/lib.rs#L196
        minicbor::to_vec(self).expect("`minicbor::to_vec` is infallible")
    }

    /// Helper function that parses a CBOR representation of an AttestationDoc and creates the
    /// structure from it, if possible.
    pub fn from_binary(bin: &[u8]) -> Result<Self> {
        minicbor::decode(bin).map_err(Error::CborDecode)
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
