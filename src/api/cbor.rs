// Copyright 2026 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! CBOR encoding and decoding on top of ciborium.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error as StdError;
use std::fmt;

/// A CBOR encoding or decoding failure.
#[derive(Debug)]
pub struct Error(Box<dyn StdError + Send + Sync>);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CBOR error: {}", self.0)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.0.as_ref())
    }
}

/// Encodes `value` as one CBOR item.
pub fn to_vec<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::new();
    ciborium::ser::into_writer(value, &mut buf).map_err(|e| Error(Box::new(e)))?;
    Ok(buf)
}

/// Decodes exactly one CBOR item. Trailing bytes are an error.
pub fn from_slice<T: DeserializeOwned>(mut bytes: &[u8]) -> Result<T, Error> {
    let value = ciborium::de::from_reader(&mut bytes).map_err(|e| Error(Box::new(e)))?;
    if !bytes.is_empty() {
        return Err(Error(
            format!("{} trailing byte(s) after CBOR item", bytes.len()).into(),
        ));
    }
    Ok(value)
}
