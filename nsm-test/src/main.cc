// Copyright 2019-2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// Author: Andrei Trandafir <aatrand@amazon.com>

#include <cstdio>
#include <cstring>
#include <vector>
#include <unistd.h>

extern "C" {
#include "../../target/debug/nsm.h"
}

// Structure holding PCR status.
typedef struct {
    bool lock;
    std::vector<uint8_t> data;
} PcrData;

// Get a string with the status of an operation
const char *get_status_string(ErrorCode status)
{
    switch (status) {
    case ERROR_CODE_SUCCESS:
        return "Success";

    case ERROR_CODE_INVALID_ARGUMENT:
        return "Invalid argument";

    case ERROR_CODE_INVALID_INDEX:
        return "Invalid index";

    case ERROR_CODE_INVALID_RESPONSE:
        return "Invalid response";

    case ERROR_CODE_READ_ONLY_INDEX:
        return "Read-only index";

    case ERROR_CODE_INVALID_OPERATION:
        return "Invalid operation";

    case ERROR_CODE_BUFFER_TOO_SMALL:
        return "Buffer too small";

    case ERROR_CODE_INPUT_TOO_LARGE:
        return "Input too large";

    case ERROR_CODE_INTERNAL_ERROR:
        return "Internal error";
    }

    return "Unknown status";
}

// Get and validate NSM description.
void get_nsm_description(int32_t ctx, NsmDescription &description)
{
    char module_str[sizeof(description.module_id) + 1];
    ErrorCode status;

    // Get NSM description.
    status = nsm_get_description(ctx, &description);
    if (status != ERROR_CODE_SUCCESS) {
        fprintf(stderr, "[Error] Request::DescribeNSM got invalid response: %s\n",
            get_status_string(status));
        exit(-1);
    }

    // The NSM must have exactly 32 PCRs.
    if (description.max_pcrs != 32) {
        fprintf(stderr, "[Error] NSM PCR count is %u.\n", description.max_pcrs);
        exit(-1);
    }

    // Convert the NSM module id to a string.
    memset(module_str, 0, sizeof(module_str));
    memcpy(module_str, description.module_id, description.module_id_len);

    // The NSM module id must not be empty.
    if (strlen(module_str) == 0) {
        fprintf(stderr, "[Error] NSM module ID is missing.\n");
        exit(-1);
    }

    // Print the NSM description.
    printf("NSM Description: [major: %u, minor: %u, patch: %u, module_id: %s, "
        "max_pcrs: %u, locked_pcrs: {",
        description.version_major, description.version_minor,
        description.version_patch, module_str, description.max_pcrs);

    // Print the list of locked PCRs.
    if (description.locked_pcrs_len > 0) {
        for (int i = 0; i < description.locked_pcrs_len - 1; ++i)
            printf("%u, ", description.locked_pcrs[i]);
        printf("%u", description.locked_pcrs[description.locked_pcrs_len - 1]);
    }

    // Print the digest type.
    printf("}, digest: ");

    switch (description.digest) {
    case DIGEST_SHA256:
        printf("SHA256].\n");
        break;

    case DIGEST_SHA384:
        printf("SHA384].\n");
        break;

    case DIGEST_SHA512:
        printf("SHA512].\n");
        break;
    }
}

// Perform and validate a single attestation operation.
void check_single_attestation(int32_t ctx,
    uint8_t *user_data, uint32_t user_data_len,
    uint8_t *nonce, uint32_t nonce_len,
    uint8_t *public_key, uint32_t public_key_len)
{
    uint8_t att_doc[16384];
    uint32_t att_doc_len = sizeof(att_doc);
    ErrorCode status;

    // Perform the attestation operation.
    status = nsm_get_attestation_doc(ctx, nonce, nonce_len,
        public_key, public_key_len, user_data, user_data_len,
        att_doc, &att_doc_len);

    if (status != ERROR_CODE_SUCCESS) {
        fprintf(stderr, "[Error] Request::Attestation got invalid response: %s\n",
            get_status_string(status));
        exit(-1);
    }

    // The received document must not be empty.
    if (att_doc_len == 0) {
        fprintf(stderr, "[Error] Attestation document is empty.\n");
        exit(-1);
    }
}

// Get the length of a PCR based on the digest type.
size_t get_pcr_len(NsmDescription &description)
{
    switch (description.digest) {
    case DIGEST_SHA256:
        return 32;

    case DIGEST_SHA384:
        return 48;

    case DIGEST_SHA512:
        return 64;

    default:
        fprintf(stderr, "[Error] Unknown PCR length.\n");
        exit(-1);
    }

    return 0;
}

// Get and validate the description of a PCR.
void get_pcr_description(int32_t ctx, int16_t index, size_t expected_pcr_len, PcrData &pcr_data)
{
    uint32_t pcr_data_len = expected_pcr_len;
    ErrorCode status;

    pcr_data.data.resize(expected_pcr_len);

    status = nsm_describe_pcr(ctx, index, &pcr_data.lock,
        pcr_data.data.data(), &pcr_data_len);
    if (status != ERROR_CODE_SUCCESS) {
        fprintf(stderr, "[Error] Request::DescribePCR got invalid response: %s\n",
            get_status_string(status));
        exit(-1);
    }

    if (pcr_data_len != expected_pcr_len) {
        fprintf(stderr, "[Error] Request::DescribePCR got invalid response length.\n");
        exit(-1);
    }
}

// Check the initial state of the PCRs.
void check_initial_pcrs(int32_t ctx, NsmDescription &description)
{
    size_t expected_pcr_len = get_pcr_len(description);
    std::vector<uint8_t> zeroed_pcr(expected_pcr_len, 0);
    std::vector<PcrData> pcr_data;

    // Get the descriptions of all PCRs.
    for (uint16_t index = 0; index < description.max_pcrs; ++index) {
        PcrData single_data;

        get_pcr_description(ctx, index, expected_pcr_len, single_data);
        pcr_data.push_back(single_data);
    }

    printf("Checked Request::DescribePCR for PCRs [0..%u).\n", description.max_pcrs);

    // PCRs [0..3) must not be empty (shund contain non-zero bytes).
    for (uint16_t index = 0; index < 3; ++index)
        if (pcr_data[index].data == zeroed_pcr) {
            fprintf(stderr, "[Error] PCR %u must not be empty.\n", index);
            exit(-1);
        }

    printf("Checked that PCRs [0..3) are not empty.\n");

    // All other PCRs should be empty.
    for (uint16_t index = 3; index < description.max_pcrs; ++index) {
        if (index == 4) {
	    // PCR4 is mapped to the parent instance-id and is not null
	    if (pcr_data[index].data == zeroed_pcr) {
                fprintf(stderr, "[Error] PCR %u must not be empty.\n", index);
                exit(-1);
            }
            continue;
        }
	if (pcr_data[index].data != zeroed_pcr) {
            fprintf(stderr, "[Error] PCR %u must be empty.\n", index);
            exit(-1);
        }
    }

    printf("Checked that PCRs [3..%u) are empty.\n", description.max_pcrs);

    // PCRs [0..16) should all be locked.
    if (description.locked_pcrs_len != 16) {
        fprintf(stderr, "[Error] Initial locked PCR list is invalid.\n");
        exit(-1);
    }

    // The list of locked PCRs from the NSM description should match [0..16).
    for (uint16_t index = 0; index < 16; ++index)
        if (description.locked_pcrs[index] != index) {
            fprintf(stderr, "[Error] Initial locked PCR list is invalid.\n");
            exit(-1);
        }

    // The PCRs [0..16) themselves should report being locked.
    for (uint16_t index = 0; index < 16; ++index)
        if (!pcr_data[index].lock) {
            fprintf(stderr, "[Error] PCR %u must be locked.\n", index);
            exit(-1);
        }

    // The rest of the PCRs should all be unlocked.
    for (uint16_t index = 16; index < description.max_pcrs; ++index)
        if (pcr_data[index].lock) {
            fprintf(stderr, "[Error] PCR %u must not be locked.\n", index);
            exit(-1);
        }

    printf("Checked that PCRs [0..16) are locked and [16..%u) are not locked.\n",
        description.max_pcrs);
}

// Check PCR locking.
void check_pcr_locks(int32_t ctx, NsmDescription &description)
{
    size_t expected_pcr_len = get_pcr_len(description);
    std::vector<uint8_t> zeroed_pcr(expected_pcr_len, 0);
    std::vector<uint8_t> dummy_data{1, 2, 3};
    uint16_t range = description.max_pcrs;
    ErrorCode status;

    // Test that PCRs [0..16) cannot be locked.
    for (uint16_t index = 0; index < 16; ++index) {
        status = nsm_lock_pcr(ctx, index);
        if (status == ERROR_CODE_SUCCESS) {
            fprintf(stderr, "[Error] PCR %u expected to not be lockable, but got: %s\n",
                index, get_status_string(status));
            exit(-1);
        }
    }

    printf("Checked Request::LockPCR for PCRs [0..16).\n");

    // Extend all unlocked PCRs multiple times with the same input.
    for (uint16_t loop_idx = 0; loop_idx < 10; ++loop_idx) {
        for (uint16_t index = 16; index < description.max_pcrs; ++index) {
            std::vector<uint8_t> pcr_data(expected_pcr_len, 0);
            uint32_t pcr_data_len = expected_pcr_len;

            // Perform PCR extension.
            status = nsm_extend_pcr(ctx, index, dummy_data.data(), dummy_data.size(),
                    pcr_data.data(), &pcr_data_len);
            if (status != ERROR_CODE_SUCCESS) {
                fprintf(stderr, "[Error] Request::ExtendPCR got invalid response: %s\n",
                    get_status_string(status));
                exit(-1);
            }

            if (pcr_data_len != expected_pcr_len) {
                fprintf(stderr, "[Error] Request::ExtendPCR got invalid response.\n");
                exit(-1);
            }

            // The extended PCR's data should not be empty.
            if (pcr_data == zeroed_pcr) {
                fprintf(stderr, "[Error] PCR %u must not be empty.\n", index);
                exit(-1);
            }
        }

        printf("[Loop: %u] Checked Request::ExtendedPCR for PCRs [16 ..%u).\n",
            loop_idx, description.max_pcrs);
    }

    // Lock all remaining PCRs.
    for (uint16_t index = 16; index < description.max_pcrs; ++index) {
        status = nsm_lock_pcr(ctx, index);
        if (status != ERROR_CODE_SUCCESS) {
            fprintf(stderr, "[Error] Request::LockPCR got invalid response: %s\n",
                get_status_string(status));
            exit(-1);
        }
    }

    printf("Checked Request::LockPCR for PCRs [16 ..%u).\n", description.max_pcrs);

    // Lock PCRs in a valid range.
    status = nsm_lock_pcrs(ctx, range);
    if (status != ERROR_CODE_SUCCESS) {
        fprintf(stderr, "[Error] Request::LockPCRs expected to succeed for [0..%u), but got: %s\n",
            range, get_status_string(status));
        exit(-1);
    }

    // Lock PCRs in an invalid range.
    ++range;
    status = nsm_lock_pcrs(ctx, range);
    if (status == ERROR_CODE_SUCCESS) {
        fprintf(stderr, "[Error] Request::LockPCRs expected to fail for [0..%u), but got: %s\n",
            range, get_status_string(status));
        exit(-1);
    }

    printf("Checked Request::LockPCRs for ranges %u and %u.\n", range - 1, range);

    // Attempt to extend the locked PCRs, which must fail.
    for (uint16_t index = 0; index < description.max_pcrs; ++index) {
        std::vector<uint8_t> pcr_data(expected_pcr_len, 0);
        uint32_t pcr_data_len = expected_pcr_len;

        status = nsm_extend_pcr(ctx, index, dummy_data.data(), dummy_data.size(),
                pcr_data.data(), &pcr_data_len);
        if (status == ERROR_CODE_SUCCESS) {
            fprintf(stderr, "[Error] Request::ExtendPCR expected to fail, but got: %s\n",
                get_status_string(status));
            exit(-1);
        }
    }

    printf("Checked Request::ExtendPCR for locked PCRs [0..%u).\n", description.max_pcrs);

    // Get the description of all PCRs multiple times.
    for (uint16_t loop_idx = 0; loop_idx < 10; ++loop_idx) {
        for (uint16_t index = 0; index < description.max_pcrs; ++index) {
            PcrData single_data;

            get_pcr_description(ctx, index, expected_pcr_len, single_data);

            // At this point, all PCRs should be locked.
            if (!single_data.lock) {
                fprintf(stderr, "[Error] PCR %u must be locked.\n", index);
                exit(-1);
            }

            // PCRs [3..16) / {4} should be empty.
            if (((index > 4) && (index < 16)) || index == 3) {
                if (single_data.data != zeroed_pcr) {
                    fprintf(stderr, "[Error] PCR %u must be empty.\n", index);
                    exit(-1);
                }
            } else {
                // All other PCRs should not be empty.
                if (single_data.data == zeroed_pcr) {
                    fprintf(stderr, "[Error] PCR %u must not be empty.\n", index);
                    exit(-1);
                }
            }
        }

        printf("[Loop: %u] Checked Request::DescribePCR for PCRs [0..%u).\n", loop_idx, description.max_pcrs);
    }
}

// Validate attestation operations
void check_attestation(int32_t ctx)
{
    const size_t DATA_LEN = 1024;
    std::vector<uint8_t> dummy_data(DATA_LEN, 128);

    // Check attestation with no input.
    check_single_attestation(ctx, NULL, 0, NULL, 0, NULL, 0);
    printf("Checked Request::Attestation without any data.\n");

    // Check attestation with only user data.
    check_single_attestation(ctx, dummy_data.data(), DATA_LEN, NULL, 0, NULL, 0);
    printf("Checked Request::Attestation with user data (%lu bytes).\n", DATA_LEN);

    // Check attestation with user data and nonce.
    check_single_attestation(ctx, dummy_data.data(), DATA_LEN, dummy_data.data(), DATA_LEN, NULL, 0);
    printf("Checked Request::Attestation with user data and nonce (%lu bytes each).\n", DATA_LEN);

    // Check attestation with user data, nonce and public key.
    check_single_attestation(ctx, dummy_data.data(), DATA_LEN, dummy_data.data(), DATA_LEN, dummy_data.data(), DATA_LEN);
    printf("Checked Request::Attestation with user data, nonce and public key (%lu bytes each).\n", DATA_LEN);
}

void check_random(int32_t ctx)
{
    const size_t DATA_LEN = 256;
    const size_t ITER_NUM = 16;
    size_t data_len;
    ErrorCode status;
        std::vector<uint8_t> dummy_data(DATA_LEN, 0);

    for (size_t i = 0; i < ITER_NUM; i++) {
        auto dummy_data_clone(dummy_data);
        data_len = DATA_LEN;
        status = nsm_get_random(ctx, dummy_data.data(), &data_len);
        if (status != ERROR_CODE_SUCCESS) {
                fprintf(stderr, "GetRandom: Got response: %s", get_status_string(status));
        }
        if (data_len != DATA_LEN) {
                fprintf(stderr, "GetRandom: Expected %zu bytes, but got %zu instead", DATA_LEN, data_len);
        }
        if (dummy_data_clone == dummy_data) {
                fprintf(stderr, "GetRandom: Got the same random bytes twice", DATA_LEN, data_len);
        }
    }
}

int main(void)
{
    NsmDescription description;
    int32_t ctx = 0;

    printf("NSM test started.\n");

    // The device file "/dev/nsm" must be opened successfully.
    ctx = nsm_lib_init();
    if (ctx < 0) {
        fprintf(stderr, "[Error] NSM initialization returned %d.", ctx);
        exit(-1);
    }

    get_nsm_description(ctx, description);

    check_single_attestation(ctx, NULL, 0, NULL, 0, NULL, 0);
    printf("Checked Request::Attestation without any data.\n");

    check_initial_pcrs(ctx, description);
    check_pcr_locks(ctx, description);
    check_attestation(ctx);

    check_random(ctx);

    nsm_lib_exit(ctx);

    printf("NSM test finished.\n");

    return 0;
}
