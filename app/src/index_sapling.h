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

#include <stdint.h>

#define T_IN_INPUT_LEN              54 //fixme: maybe add more options to HDPATH
#define T_OUT_INPUT_LEN             34
#define SPEND_INPUT_LEN             55
#define OUTPUT_INPUT_LEN            85

#define INDEX_INPUT_TIN_PATH        0
#define INDEX_INPUT_TIN_SCRIPT      20
#define INDEX_INPUT_TIN_VALUE       46

#define INDEX_INPUT_TOUT_ADDR       0
#define INDEX_INPUT_TOUT_VALUE      26

#define INDEX_INPUT_SPENDPOS        0
#define INDEX_INPUT_INPUTDIV        4
#define INDEX_INPUT_INPUTPKD        15
#define INDEX_INPUT_INPUTVALUE      47

#define INDEX_INPUT_OUTPUTDIV       0
#define INDEX_INPUT_OUTPUTPKD       11
#define INDEX_INPUT_OUTPUTVALUE     43
#define INDEX_INPUT_OUTPUTMEMO      51
#define INDEX_INPUT_OUTPUTOVK       52

#define SPEND_EXTRACT_LEN           128
#define OUTPUT_EXTRACT_LEN          64

#define T_IN_TX_LEN                 74
#define SPEND_OLD_TX_LEN            40
#define SPEND_TX_LEN                320
#define OUTPUT_TX_LEN               948

#define INDEX_TIN_PREVOUT           0
#define INDEX_TIN_SCRIPT            36
#define INDEX_TIN_VALUE             62
#define INDEX_TIN_SEQ               70

#define INDEX_SPEND_OLD_RCM         0
#define INDEX_SPEND_OLD_NOTEPOS     32

#define INDEX_SPEND_RK              96
#define INDEX_SPEND_VALUECMT        0
#define INDEX_SPEND_NF              64

#define INDEX_OUTPUT_VALUECMT       0
#define INDEX_OUTPUT_NOTECMT        32
#define INDEX_OUTPUT_EPK            64
#define INDEX_OUTPUT_ENC            96
#define INDEX_OUTPUT_OUT            676

#define LENGTH_HASH_DATA                220
#define INDEX_HASH_PREVOUTSHASH         8
#define INDEX_HASH_SEQUENCEHASH         40
#define INDEX_HASH_OUTPUTSHASH          72
#define INDEX_HASH_JOINSPLITSHASH       104
#define INDEX_HASH_SHIELDEDSPENDHASH    136
#define INDEX_HASH_SHIELDEDOUTPUTHASH   168
#define INDEX_HASH_VALUEBALANCE         208

uint16_t length_t_in_data();

uint16_t length_spend_old_data();

uint16_t length_spend_new_data();

uint16_t length_spenddata();

uint16_t length_outputdata();

uint16_t start_sighashdata();
