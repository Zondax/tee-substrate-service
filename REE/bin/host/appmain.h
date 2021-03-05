#pragma once

#include <tee_client_api.h>

TEEC_Result initialize_context(TEEC_Context *ctx);
TEEC_Result open_session(TEEC_Context *ctx, TEEC_Session *sess);
void appMain(TEEC_Session *sess, TEEC_Context *ctx);
void cleanup(TEEC_Session *sess, TEEC_Context *ctx);
void recover_panic(void);
