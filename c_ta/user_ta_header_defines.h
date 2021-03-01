#pragma once

#include <rustee_ta.h>

#define TA_VERSION              "1.0"
#define TA_DESCRIPTION          "RemoTEE Signer"
#define TA_FLAGS                TA_FLAG_EXEC_DDR
#define TA_STACK_SIZE           (4 * 1024)
#define TA_DATA_SIZE            (32 * 1024)

// Extra properties
#define TA_CURRENT_TA_EXT_PROPERTIES
