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

clean: cclean
	$(MAKE) -C framework $@

cclean:
	$(MAKE) -C framework $@

run: copy
	$(MAKE) -C framework $@

run-debug: copy
	$(MAKE) -C framework $@
