#include <tee_internal_api.h>
#include <tee_internal_api_extensions.h>

#include <librustee_ta.h>
#include <rustee_ta.h>

#define UNUSED(X) (void)&X;

TEE_Result TA_OpenSessionEntryPoint(uint32_t param_types,
                                    TEE_Param params[4],
                                    void **sess_ctx) {
    UNUSED(params)
    UNUSED(sess_ctx)
    UNUSED(param_types)
    DMSG("Open Session entry point\n");

    return TEE_SUCCESS;
}

TEE_Result TA_InvokeCommandEntryPoint(void *sess_ctx,
                                      uint32_t cmd_id,
                                      uint32_t param_types,
                                      TEE_Param params[4]) {
    UNUSED(sess_ctx)

    return invoke_command(cmd_id, param_types, params);
}
