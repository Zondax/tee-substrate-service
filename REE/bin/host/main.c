#include <err.h>
#include <stdio.h>
#include <string.h>

#include <tee_client_api.h>
#include <rustee_ta.h>
#include "appmain.h"

int main(void) {
    TEEC_Result res;
    TEEC_Context ctx;
    TEEC_Session sess;
    printf("Requesting session %d\n", 1);

    uint32_t err_origin;
    res = TEEC_InitializeContext(NULL, &ctx);
    if (res != TEEC_SUCCESS) {
        errx(1, "TEEC_InitializeContext failed [Code 0x%x]", res);
    }

    const TEEC_UUID uuid = TA_UUID;
    res = TEEC_OpenSession(&ctx, &sess, &uuid, TEEC_LOGIN_PUBLIC, NULL, NULL, &err_origin);
    if (res != TEEC_SUCCESS) {
        errx(1, "TEEC_Opensession failed. [Code 0x%x origin 0x%x]", res, err_origin);
    }

    // Now call the rust app passing the session
    appMain(&sess);
    printf("Clossing session");

    // Close session and context
    TEEC_CloseSession(&sess);
    TEEC_FinalizeContext(&ctx);

    return 0;
}
