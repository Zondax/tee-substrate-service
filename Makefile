.PHONY: all clean copy cclean run run-debug deps
default: all

include proj.mk

deps:
	rustup target add $(RUST_TARGET)

all: cclean
	$(MAKE) -C $(HOST) all
	$(MAKE) -C $(TA) all

copy: all
	$(MAKE) -C $(TA) copy
	$(MAKE) -C $(HOST) copy

clean:
	$(MAKE) -C $(HOST) clean
	$(MAKE) -C $(TA) clean

cclean:
	$(MAKE) -C $(HOST) cclean
	$(MAKE) -C $(TA) cclean

run: copy
	$(MAKE) -C $(OPTEE)/build run-only QEMU_VIRTFS_ENABLE=y QEMU_VIRTFS_HOST_DIR=$(SHARED_FOLDER)

run-debug: copy
	$(MAKE) -C $(OPTEE)/build run-only GDBSERVER=Y CFG_TEE_CORE_LOG_LEVEL=4 CFG_CORE_ASLR=n CFG_TA_ASLR=n CFG_TEE_TA_LOG_LEVEL=4 QEMU_VIRTFS_ENABLE=y QEMU_VIRTFS_HOST_DIR=$(SHARED_FOLDER)
