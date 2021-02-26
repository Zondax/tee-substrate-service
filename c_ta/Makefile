include include/uuid.mk

BINARY=$(TA_UUID)

CFG_TEE_TA_LOG_LEVEL ?= 4
CFG_TEE_CORE_LOG_LEVEL ?= 4
# TODO: CFG_ENCRYPT_TA
# TODO: TA_ENC_KEY

CPPFLAGS += -DCFG_TEE_TA_LOG_LEVEL=$(CFG_TEE_TA_LOG_LEVEL) -DCFG_TEE_CORE_LOG_LEVEL=$(CFG_TEE_CORE_LOG_LEVEL)
-include $(TA_DEV_KIT_DIR)/mk/ta_dev_kit.mk

ifeq ($(wildcard $(TA_DEV_KIT_DIR)/mk/ta_dev_kit.mk), )
clean:
	@echo 'Note: $$(TA_DEV_KIT_DIR)/mk/ta_dev_kit.mk not found, cannot clean TA'
endif

new-uuid:
	python newuuid.py