/*******************************************************************************
*   (c) 2018 -2022 Zondax AG
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/

#pragma once

#include "inttypes.h"
#include "zxmacros.h"

// Based on ISO7816

#define APDU_CODE_OK                        0x9000
#define APDU_CODE_BUSY                      0x9001

#define APDU_CODE_EXECUTION_ERROR           0x6400

#define APDU_CODE_WRONG_LENGTH              0x6700

#define APDU_CODE_EMPTY_BUFFER              0x6982
#define APDU_CODE_OUTPUT_BUFFER_TOO_SMALL   0x6983
#define APDU_CODE_DATA_INVALID              0x6984
#define APDU_CODE_CONDITIONS_NOT_SATISFIED  0x6985
#define APDU_CODE_COMMAND_NOT_ALLOWED       0x6986
#define APDU_CODE_TX_NOT_INITIALIZED        0x6987
#define APDU_CODE_DATA_TOO_LONG             0x6988
#define APDU_CODE_EXTRACT_TRANSACTION_FAIL  0x6989
#define APDU_CODE_HASH_MSG_BUF_FAIL         0x6990
#define APDU_CODE_UNPROCESSED_TX            0x6991
#define APDU_CODE_PREVOUT_INVALID           0x6992
#define APDU_CODE_SEQUENCE_INVALID          0x6993
#define APDU_CODE_OUTPUTS_INVALID           0x6994
#define APDU_CODE_JOINSPLIT_INVALID         0x6995
#define APDU_CODE_SPEND_INVALID             0x6996
#define APDU_CODE_OUTPUT_CONTENT_INVALID    0x6997
#define APDU_CODE_ENCRYPTION_INVALID        0x6998
#define APDU_CODE_CHECK_SIGN_TR_FAIL        0x6999
#define APDU_SIGN_SPEND_FAIL                0x69A0
#define APDU_CODE_BAD_VALUEBALANCE          0x69A1

#define APDU_CODE_BAD_KEY_HANDLE            0x6A80
#define APDU_CODE_INVALIDP1P2               0x6B00
#define APDU_CODE_INS_NOT_SUPPORTED         0x6D00
#define APDU_CODE_CLA_NOT_SUPPORTED         0x6E00

#define APDU_CODE_UNKNOWN                   0x6F00
#define APDU_CODE_SIGN_VERIFY_ERROR         0x6F01


__Z_INLINE void set_code(uint8_t *buffer, uint8_t offset, uint16_t value) {
    *(buffer + offset) = (uint8_t) (value >> 8);
    *(buffer + offset + 1) = (uint8_t) (value & 0xFF);
}
