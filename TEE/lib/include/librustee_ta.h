#pragma once

#include <stdint.h>

extern uint32_t invoke_command(uint32_t cmd_id, uint32_t param_types, TEE_Param params[4]);