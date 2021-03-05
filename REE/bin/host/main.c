#include <err.h>
#include <stdio.h>
#include <string.h>

#include "appmain.h"
#include <rustee_ta.h>
#include <tee_client_api.h>

int main(void) {
  TEEC_Context ctx;
  TEEC_Session sess;
  printf("Requesting session\n");

  // Create a context and open a session with that context
  initialize_context(&ctx);
  open_session(&ctx, &sess);

  // Now call the rust app passing the session
  appMain(&sess, &ctx);
  printf("Clossing session");

  // Close session and context
  cleanup(&sess, &ctx);

  return 0;
}
