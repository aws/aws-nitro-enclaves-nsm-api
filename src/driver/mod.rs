// Copyright 2020-2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! ***NitroSecureModule driver communication support***
//! # Overview
//! This module implements support functions for communicating with the NSM
//! driver by encoding requests to / decoding responses from a C-compatible
//! message structure which is shared with the driver via `ioctl()`.
//! In general, a message contains:
//! 1. A *request* content structure, holding CBOR-encoded user input data.
//! 2. A *response* content structure, with an initial memory capacity provided by
//! the user, which then gets populated with information from the NSM driver and
//! then decoded from CBOR.

use crate::api::{ErrorCode, Request, Response};
use libc::ioctl;
use log::{debug, error};
use nix::errno::Errno;
use nix::request_code_readwrite;
use nix::sys::uio::IoVec;
use nix::unistd::close;

use std::fs::OpenOptions;
use std::mem;
use std::os::unix::io::{IntoRawFd, RawFd};

const DEV_FILE: &str = "/dev/nsm";
const NSM_IOCTL_MAGIC: u8 = 0x0A;
const NSM_REQUEST_MAX_SIZE: usize = 0x1000;
const NSM_RESPONSE_MAX_SIZE: usize = 0x3000;

/// NSM message structure to be used with `ioctl()`.
#[repr(C)]
struct NsmMessage<'a> {
    /// User-provided data for the request
    pub request: IoVec<&'a [u8]>,
    /// Response data provided by the NSM pipeline
    pub response: IoVec<&'a mut [u8]>,
}

/// Encode an NSM `Request` value into a vector.  
/// *Argument 1 (input)*: The NSM request.  
/// *Returns*: The vector containing the CBOR encoding.
fn nsm_encode_request_to_cbor(request: Request) -> Vec<u8> {
    serde_cbor::to_vec(&request).unwrap()
}

/// Decode an NSM `Response` value from a raw memory buffer.  
/// *Argument 1 (input)*: The `iovec` holding the memory buffer.  
/// *Returns*: The decoded NSM response.
fn nsm_decode_response_from_cbor(response_data: &IoVec<&mut [u8]>) -> Response {
    match serde_cbor::from_slice(response_data.as_slice()) {
        Ok(response) => response,
        Err(_) => Response::Error(ErrorCode::InternalError),
    }
}

/// Do an `ioctl()` of a given type for a given message.  
/// *Argument 1 (input)*: The descriptor to the device file.  
/// *Argument 2 (input/output)*: The message to be sent and updated via `ioctl()`.  
/// *Returns*: The status of the operation.
fn nsm_ioctl(fd: i32, message: &mut NsmMessage) -> Option<Errno> {
    let status = unsafe {
        ioctl(
            fd,
            request_code_readwrite!(NSM_IOCTL_MAGIC, 0, mem::size_of::<NsmMessage>()),
            message,
        )
    };
    let errno = Errno::last();

    match status {
        // If ioctl() succeeded, the status is the message's response code
        0 => None,

        // If ioctl() failed, the error is given by errno
        _ => Some(errno),
    }
}

/// Create a message with input data and output capacity from a given
/// request, then send it to the NSM driver via `ioctl()` and wait
/// for the driver's response.  
/// *Argument 1 (input)*: The descriptor to the NSM device file.  
/// *Argument 2 (input)*: The NSM request.  
/// *Returns*: The corresponding NSM response from the driver.
pub fn nsm_process_request(fd: i32, request: Request) -> Response {
    let cbor_request = nsm_encode_request_to_cbor(request);

    // Check if the request is too large
    if cbor_request.len() > NSM_REQUEST_MAX_SIZE {
        return Response::Error(ErrorCode::InputTooLarge);
    }

    let mut cbor_response: [u8; NSM_RESPONSE_MAX_SIZE] = [0; NSM_RESPONSE_MAX_SIZE];
    let mut message = NsmMessage {
        request: IoVec::from_slice(&cbor_request),
        response: IoVec::from_mut_slice(&mut cbor_response),
    };
    let status = nsm_ioctl(fd, &mut message);

    match status {
        None => nsm_decode_response_from_cbor(&message.response),
        Some(errno) => match errno {
            Errno::EMSGSIZE => Response::Error(ErrorCode::InputTooLarge),
            _ => Response::Error(ErrorCode::InternalError),
        },
    }
}

/// NSM library initialization function.  
/// *Returns*: A descriptor for the opened device file.
pub fn nsm_init() -> i32 {
    let mut open_options = OpenOptions::new();
    let open_dev = open_options.read(true).write(true).open(DEV_FILE);

    match open_dev {
        Ok(open_dev) => {
            debug!("Device file '{}' opened successfully.", DEV_FILE);
            open_dev.into_raw_fd() as i32
        }
        Err(e) => {
            error!("Device file '{}' failed to open: {}", DEV_FILE, e);
            -1
        }
    }
}

/// NSM library exit function.  
/// *Argument 1 (input)*: The descriptor for the opened device file, as
/// obtained from `nsm_init()`.
pub fn nsm_exit(fd: i32) {
    let result = close(fd as RawFd);
    match result {
        Ok(()) => debug!("File of descriptor {} closed successfully.", fd),
        Err(e) => error!("File of descriptor {} failed to close: {}", fd, e),
    }
}
