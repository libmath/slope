MAKEFILE_PATH   := $(abspath $(lastword $(MAKEFILE_LIST)))
MAKEFILE_DIR    := $(dir $(MAKEFILE_PATH))

LEAN_DIR        := $(MAKEFILE_DIR)..
CARGO_DEBUG_DIR := $(MAKEFILE_DIR)target/debug

CARGO_INSTALL := cargo install --locked --all-features --force

BIN_NAME := slope

current: search

build:
	cargo build
	cd $(LEAN_DIR) && PATH=$(CARGO_DEBUG_DIR):$$PATH $(BIN_NAME) build

search:
	cargo build
	cd $(LEAN_DIR) && PATH=$(CARGO_DEBUG_DIR):$$PATH $(BIN_NAME) search

graph:
	cargo build
	cd $(LEAN_DIR) && PATH=$(CARGO_DEBUG_DIR):$$PATH $(BIN_NAME) graph

rg:
	cargo build
	cd $(LEAN_DIR) && PATH=$(CARGO_DEBUG_DIR):$$PATH $(BIN_NAME) rg

check-fmt:
	cargo build
	cd $(LEAN_DIR) && PATH=$(CARGO_DEBUG_DIR):$$PATH $(BIN_NAME) check-fmt

install:
	$(CARGO_INSTALL) --path .
