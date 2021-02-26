global-incdirs-y += include ../$(RUST_TA)/include
srcs-y += rustee_ta.c

libnames += rustee_ta
# Add both search paths
libdirs	+= ../target/armv7-unknown-linux-gnueabihf/release
libdirs	+= ../target/aarch64-unknown-linux-gnu/release
