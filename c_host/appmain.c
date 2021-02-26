#include "appmain.h"
#include <err.h>
#include <stdio.h>
#include <string.h>

#include <librustee_host.h>
#include <rustee_ta.h>

// The optee session to be used along the program execution
TEEC_Session *session = NULL;

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

void appMain(TEEC_Session *sess) {
  // TODO: Eventually this should be a rust function
  // First we need to expose types and some functions such as TEEC_InvokeCommand
  session = sess;

  call_rustee(sess);
}
