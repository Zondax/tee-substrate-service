.PHONY: all cclean clean copy
default: all

include ../proj.mk

all:
	cargo build --target $(RUST_TARGET) --release
	$(MAKE) -C bin/host CROSS_COMPILE="$(HOST_CROSS_COMPILE)" --no-builtin-variables
#TODO: move files to ./out dir

clean: cclean
	cargo clean
	rm $(SHARED_FOLDER)/$(PROJ_NAME)

cclean:
	$(MAKE) -C bin/host clean

copy: all
	cp bin/host/$(PROJ_NAME) $(SHARED_FOLDER)
