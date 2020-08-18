/*
 * Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#ifndef NSM_LIB_H
#define NSM_LIB_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * The digest implementation used by a NitroSecureModule
 */
typedef enum {
    /**
     * SHA256
     */
    DIGEST_SHA256,
    /**
     * SHA384
     */
    DIGEST_SHA384,
    /**
     * SHA512
     */
    DIGEST_SHA512,
} Digest;

/**
 * List of error codes that the NSM module can return as part of a Response
 */
typedef enum {
    /**
     * No errors
     */
    ERROR_CODE_SUCCESS,
    /**
     * Input argument(s) invalid
     */
    ERROR_CODE_INVALID_ARGUMENT,
    /**
     * PlatformConfigurationRegister index out of bounds
     */
    ERROR_CODE_INVALID_INDEX,
    /**
     * The received response does not correspond to the earlier request
     */
    ERROR_CODE_INVALID_RESPONSE,
    /**
     * PlatformConfigurationRegister is in read-only mode and the operation
     * attempted to modify it
     */
    ERROR_CODE_READ_ONLY_INDEX,
    /**
     * Given request cannot be fulfilled due to missing capabilities
     */
    ERROR_CODE_INVALID_OPERATION,
    /**
     * Operation succeeded but provided output buffer is too small
     */
    ERROR_CODE_BUFFER_TOO_SMALL,
    /**
     * The user-provided input is too large
     */
    ERROR_CODE_INPUT_TOO_LARGE,
    /**
     * NitroSecureModule cannot fulfill request due to internal errors
     */
    ERROR_CODE_INTERNAL_ERROR,
} ErrorCode;

typedef struct {
    uint16_t version_major;
    uint16_t version_minor;
    uint16_t version_patch;
    uint8_t module_id[100];
    uint32_t module_id_len;
    uint16_t max_pcrs;
    uint16_t locked_pcrs[64];
    uint32_t locked_pcrs_len;
    Digest digest;
} NsmDescription;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * NSM `DescribePCR` operation for non-Rust callers.
 * *Argument 1 (input)*: The descriptor to the NSM device file.
 * *Argument 2 (input)*: The index of the PCR to be described.
 * *Argument 3 (output)*: The lock state of the PCR.
 * *Argument 4 (output)*: The buffer that will hold the PCR data.
 * *Argument 5 (input / output)*: The PCR data buffer capacity (as input)
 * and the actual size of the received data (as output).
 * *Returns*: The status of the operation.
 */
ErrorCode nsm_describe_pcr(int32_t fd,
                           uint16_t index,
                           bool *lock,
                           uint8_t *data,
                           uint32_t *data_len);

/**
 * NSM `ExtendPCR` operation for non-Rust callers.
 * *Argument 1 (input)*: The descriptor to the NSM device file.
 * *Argument 2 (input)*: The index of the PCR to extend.
 * *Argument 3 (input)*: The raw data to extend the PCR with.
 * *Argument 4 (input)*: The length of the raw data, in bytes.
 * *Argument 5 (output)*: The data from the extended PCR.
 * *Argument 6 (input/output)*: The capacity of the extended PCR data
 * buffer as input, the actual size of the buffer as output.
 * *Returns*: The status of the operation.
 */
ErrorCode nsm_extend_pcr(int32_t fd,
                         uint16_t index,
                         const uint8_t *data,
                         uint32_t data_len,
                         uint8_t *pcr_data,
                         uint32_t *pcr_data_len);

/**
 * NSM `GetAttestationDoc` operation for non-Rust callers.
 * *Argument 1 (input)*: The descriptor to the NSM device file.
 * *Argument 2 (input)*: User data.
 * *Argument 3 (input)*: The size of the user data buffer.
 * *Argument 4 (input)*: Nonce data.
 * *Argument 5 (input)*: The size of the nonce data buffer.
 * *Argument 6 (input)*: Public key data.
 * *Argument 7 (input)*: The size of the public key data buffer.
 * *Argument 8 (output)*: The obtained attestation document.
 * *Argument 9 (input / output)*: The document buffer capacity (as input)
 * and the size of the received document (as output).
 * *Returns*: The status of the operation.
 */
ErrorCode nsm_get_attestation_doc(int32_t fd,
                                  const uint8_t *user_data,
                                  uint32_t user_data_len,
                                  const uint8_t *nonce_data,
                                  uint32_t nonce_len,
                                  const uint8_t *pub_key_data,
                                  uint32_t pub_key_len,
                                  uint8_t *att_doc_data,
                                  uint32_t *att_doc_len);

/**
 * NSM `Describe` operation for non-Rust callers.
 * *Argument 1 (input)*: The descriptor to the NSM device file.
 * *Argument 2 (output)*: The obtained raw NSM description.
 * *Returns*: The status of the operation.
 */
ErrorCode nsm_get_description(int32_t fd, NsmDescription *nsm_description);

/**
 * NSM `GetRandom` operation for non-Rust callers. Returns up to 256 bytes of random data.
 * *fd (input)*: A valid descriptor to the NSM device.
 * *buf (output)*: A valid buffer to place the random data in.
 * *buf_len (input / output)*: The length of the passed buffer and the length of the output data
 *                             if the function finishes with ErrorCode::Success.
 */
ErrorCode nsm_get_random(int32_t fd, uint8_t *buf, uintptr_t *buf_len);

/**
 * NSM library exit function.
 * *Argument 1 (input)*: The descriptor for the opened device file, as
 * obtained from `nsm_init()`.
 */
void nsm_lib_exit(int32_t fd);

/**
 * NSM library initialization function.
 * *Returns*: A descriptor for the opened device file.
 */
int32_t nsm_lib_init(void);

/**
 * NSM `LockPCR` operation for non-Rust callers.
 * *Argument 1 (input)*: The descriptor to the NSM device file.
 * *Argument 2 (input)*: The PCR to be locked.
 * *Returns*: The status of the operation.
 */
ErrorCode nsm_lock_pcr(int32_t fd, uint16_t index);

/**
 * NSM `LockPCRs` operation for non-Rust callers.
 * *Argument 1 (input)*: The descriptor to the NSM device file.
 * *Argument 2 (input)*: The range value for `[0, range)` to be locked.
 * *Returns*: The status of the operation.
 */
ErrorCode nsm_lock_pcrs(int32_t fd, uint16_t range);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* NSM_LIB_H */
