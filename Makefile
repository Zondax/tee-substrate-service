.PHONY: all clean copy cclean run run-debug deps
default: all

include proj.mk

ifdef QEMU_V8
.PHONY: deps-clean deps-qemu
# found inside the qemu_v8.mk for optee
deps-clean:
	rm -rf $(OPTEE)/out-br/target
	find $(OPTEE)/out-br/ -name .stamp_target_installed | xargs rm

#build qemu
deps-qemu:
	$(MAKE) -C $(OPTEE)/build -j4 \
		QEMU_VIRTFS_ENABLE=y \
		QEMU_VIRTFS_AUTOMOUNT=y \
		QEMU_VIRTFS_MOUNTPOINT=/root \
		CFG_TEE_RAM_VA_SIZE=0x00300000 \
		BR2_PACKAGE_BUSYBOX_SHOW_OTHERS=y \
		BR2_PACKAGE_NETCAT=y
endif

deps:
	rustup target add $(RUST_TARGET)

all:
	$(MAKE) -C $(HOST) all
	$(MAKE) -C $(TA) all

copy:
	$(MAKE) -C $(TA) copy
	$(MAKE) -C $(HOST) copy

clean: cclean
	$(MAKE) -C $(HOST) clean
	$(MAKE) -C $(TA) clean

cclean:
	$(MAKE) -C $(HOST) cclean
	$(MAKE) -C $(TA) cclean

run: copy
	$(MAKE) -C $(OPTEE)/build run-only

run-debug: copy
	$(MAKE) -C $(OPTEE)/build run-only GDBSERVER=y \
										CFG_TEE_CORE_LOG_LEVEL=4 \
										CFG_CORE_ASLR=n CFG_TA_ASLR=n \
										CFG_TEE_TA_LOG_LEVEL=4
