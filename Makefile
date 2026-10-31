TGT        := speedtui
TARGET_DIR := /home/foxter/.cache/speedtest-tui/target
PREFIX     ?= /usr/local
BINDIR     ?= $(PREFIX)/bin

all: $(TGT)

$(TGT): Cargo.toml src/main.rs src/dsp.rs src/engn.rs src/grph.rs
	cargo build --release
	cp --remove-destination $(TARGET_DIR)/release/$(TGT) ./$(TGT)

install: $(TGT)
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 $(TARGET_DIR)/release/$(TGT) $(DESTDIR)$(BINDIR)/$(TGT)

user-install: $(TGT)
	mkdir -p $(HOME)/.local/bin
	cp --remove-destination $(TARGET_DIR)/release/$(TGT) $(HOME)/.local/bin/$(TGT)

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(TGT)
	rm -f $(HOME)/.local/bin/$(TGT)

clean:
	cargo clean
	rm -f $(TGT) speedtest-tui

.PHONY: all install user-install uninstall clean
