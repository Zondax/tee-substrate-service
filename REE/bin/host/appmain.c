#include "appmain.h"
#include <err.h>
#include <stdio.h>
#include <string.h>

#include <librustee_host.h>
#include <rustee_ta.h>

// The optee session to be used along the program execution
TEEC_Session *session = NULL;
TEEC_Context *context = NULL;

TEEC_Result call_rustee(TEEC_Session *sess) {
  printf("running client service\n");
  run();
  return TEEC_SUCCESS;
}

TEEC_Result invoke_optee_command(uint32_t command_id, TEEC_Operation *op) {
  uint32_t err_origin = 0;
  if (session == NULL) {
    return TEEC_ERROR_ITEM_NOT_FOUND;
  }
  op->session = session;

  return TEEC_InvokeCommand(session, command_id, op, &err_origin);
}

TEEC_Result initialize_context(TEEC_Context *ctx) {
  TEEC_Result res;
  TEEC_Context ctx_;

  res = TEEC_InitializeContext(NULL, &ctx_);
  if (res != TEEC_SUCCESS) {
    errx(1, "TEEC_InitializeContext failed [Code 0x%x]", res);
  }

  *ctx = ctx_;
  return res;
}

TEEC_Result open_session(TEEC_Context *ctx, TEEC_Session *sess) {
  const TEEC_UUID uuid = TA_UUID;
  TEEC_Result res;
  TEEC_Session session;
  uint32_t err_origin;

  res = TEEC_OpenSession(ctx, &session, &uuid, TEEC_LOGIN_PUBLIC, NULL, NULL,
                         &err_origin);

  if (res != TEEC_SUCCESS) {
    errx(1, "TEEC_Opensession failed. [Code 0x%x origin 0x%x]", res,
         err_origin);
  }

  *sess = session;
  return res;
}

void cleanup(TEEC_Session *sess, TEEC_Context *ctx) {
  TEEC_CloseSession(sess);
  TEEC_FinalizeContext(ctx);
}

void recover_panic() {
  printf("TA seems to have panicked, starting new instance...\n");
  cleanup(session, context);

  initialize_context(context);
  open_session(context, session);
}

void appMain(TEEC_Session *sess, TEEC_Context *ctx) {
  // TODO: Eventually this should be a rust function
  // First we need to expose types and some functions such as TEEC_InvokeCommand
  session = sess;
  context = ctx;

  call_rustee(sess);
}
