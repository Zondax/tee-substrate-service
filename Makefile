.PHONY: all clean
default: all

include proj.mk
# Set environment according to /optee/build/common.mk & specific

ifdef QEMU_V7
$(info )
$(info ************  QEMU V7 ************)
$(info )
export RUST_TARGET ?= armv7-unknown-linux-gnueabihf
export OPTEE ?= $(HOME)/optee
export TEEC_EXPORT ?= $(OPTEE)/out-br/host/arm-buildroot-linux-gnueabihf/sysroot
export HOST_CROSS_COMPILE ?= $(OPTEE)/toolchains/aarch32/bin/arm-linux-gnueabihf-
export TA_CROSS_COMPILE ?= $(HOST_CROSS_COMPILE)
export TA_DEV_KIT_DIR ?= $(OPTEE)/optee_os/out/arm/export-ta_arm32
export OVERRIDE_SYSROOT ?= 1
endif

ifdef QEMU_V8
$(info )
$(info ************  QEMU V8 ************)
$(info )
export RUST_TARGET ?= aarch64-unknown-linux-gnu
export OPTEE ?= $(HOME)/optee
export TEEC_EXPORT ?= $(OPTEE)/out-br/host/aarch64-buildroot-linux-gnu/sysroot
export HOST_CROSS_COMPILE = $(OPTEE)/toolchains/aarch64/bin/aarch64-linux-gnu-
export TA_CROSS_COMPILE ?= $(HOST_CROSS_COMPILE)
export TA_DEV_KIT_DIR ?= $(OPTEE)/optee_os/out/arm/export-ta_arm64
export OVERRIDE_SYSROOT ?= 1

endif

export V?=0
export UTEE_ROOT=$(TA_DEV_KIT_DIR)
export TEEC_ROOT=$(TEEC_EXPORT)/usr

deps:
	rustup target add $(RUST_TARGET)

all:
	cargo build --target $(RUST_TARGET) --release
	$(MAKE) -C $(C_HOST) CROSS_COMPILE="$(HOST_CROSS_COMPILE)" --no-builtin-variables
	$(MAKE) -C $(C_TA) CROSS_COMPILE="$(TA_CROSS_COMPILE)" LDFLAGS=""

include $(C_TA)/include/uuid.mk
copy: all
	cp $(C_HOST)/$(PROJ_NAME) $(SHARED_FOLDER)
	cp $(C_TA)/$(TA_UUID).ta $(SHARED_FOLDER)

clean:
	CARGO_HOME=$(CARGO_CUSTOM_HOME) && cargo clean
	$(MAKE) -C $(C_HOST) clean
	$(MAKE) -C $(C_TA) clean
	rm $(SHARED_FOLDER)/$(PROJ_NAME) $(SHARED_FOLDER)/$(TA_UUID).ta

cclean:
	$(MAKE) -C $(C_HOST) clean
	$(MAKE) -C $(C_TA) clean

run: copy
	$(MAKE) -C $(OPTEE)/build run-only QEMU_VIRTFS_ENABLE=y QEMU_VIRTFS_HOST_DIR=$(SHARED_FOLDER)

run-debug: copy
	$(MAKE) -C $(OPTEE)/build run-only GDBSERVER=Y CFG_TEE_CORE_LOG_LEVEL=4 CFG_CORE_ASLR=n CFG_TA_ASLR=n CFG_TEE_TA_LOG_LEVEL=4 QEMU_VIRTFS_ENABLE=y QEMU_VIRTFS_HOST_DIR=$(SHARED_FOLDER)
