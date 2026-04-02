.PHONY: all install clean

CARGO ?= cargo
CARGO_FLAGS ?= --release --features lua,native

# Platform-specific library names
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    LIB_EXT := dylib
    LIB_PREFIX := lib
else ifeq ($(UNAME_S),Linux)
    LIB_EXT := so
    LIB_PREFIX := lib
else
    LIB_EXT := dll
    LIB_PREFIX :=
endif

LIB_NAME := $(LIB_PREFIX)goud_engine.$(LIB_EXT)
BUILD_DIR := ../../../target/release

all:
	cd ../../.. && $(CARGO) build $(CARGO_FLAGS)

install: all
	install -d $(INST_LIBDIR)
	install -d $(INST_LUADIR)/goudengine
	install -m 644 $(BUILD_DIR)/$(LIB_NAME) $(INST_LIBDIR)/$(LIB_NAME)
	install -m 644 goudengine/init.lua $(INST_LUADIR)/goudengine/init.lua
	install -m 644 goudengine/constants.lua $(INST_LUADIR)/goudengine/constants.lua

clean:
	cd ../../.. && $(CARGO) clean
