#pragma once

#include <stdint.h>

extern TEE_Result RUSTEE_Create(void);
extern void RUSTEE_Destroy(void);

extern TEE_Result RUSTEE_OpenSession(uint32_t param_types, TEE_Param params[4], void **sess_ctx);
extern void RUSTEE_CloseSession(void* sess_ctx);

extern TEE_Result RUSTEE_InvokeCommand(void* sess_ctx, uint32_t cmd_id, uint32_t param_types, TEE_Param params[4]);
