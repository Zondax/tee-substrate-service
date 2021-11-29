.PHONY: all clean copy cclean run run-debug deps
default: all

ifdef QEMU_V8
.PHONY: deps-clean deps-qemu
# found inside the qemu_v8.mk for optee
qemu-clean:
	$(MAKE) -C framework $@

#build qemu
qemu:
	$(MAKE) -C framework $@
endif

deps:
	git submodule update --init
	$(MAKE) -C framework $@

all:
	$(MAKE) -C framework $@

copy:
	$(MAKE) -C framework $@

ci:
	$(MAKE) -C framework $@

clean: cclean
	$(MAKE) -C framework $@

cclean:
	$(MAKE) -C framework $@

run: copy
	$(MAKE) -C framework $@

run-debug: copy
	$(MAKE) -C framework $@

fuzz_dep:
	cargo install cargo-fuzz --force

fuzz:
	cd TEE/common/ta-app && cargo fuzz list |\
	xargs -I% -n1 -P$(shell nproc) cargo fuzz run % --sanitizer=none -- \
	-detect_leaks=0 -use_value_profile=1
