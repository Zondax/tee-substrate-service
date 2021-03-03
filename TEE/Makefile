 .PHONY: all cclean clean copy
default: all

include ../proj.mk
include ./bin/ta/include/uuid.mk

all:
	cargo build --target $(RUST_TARGET) --release
	$(MAKE) -C bin/ta CROSS_COMPILE="$(TA_CROSS_COMPILE)" LDFLAGS=""
#TODO: move files to ./out dir

clean: cclean
	cargo clean
	rm $(SHARED_FOLDER)/$(TA_UUID).ta

cclean:
	$(MAKE) -C bin/ta clean

copy: all
	cp bin/ta/$(TA_UUID).ta $(SHARED_FOLDER)
