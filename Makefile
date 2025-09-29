CARGO ?= cargo
BIN_NAME ?= freegin-ai
PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
MANDIR ?= $(PREFIX)/share/man
MANPAGE ?= docs/man/$(BIN_NAME).1

.PHONY: build release run test fmt clean install uninstall

build:
	$(CARGO) build

release:
	$(CARGO) build --release

run:
	$(CARGO) run

test:
	$(CARGO) test

fmt:
	$(CARGO) fmt

clean:
	$(CARGO) clean

install: release
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 target/release/$(BIN_NAME) $(DESTDIR)$(BINDIR)/$(BIN_NAME)
	install -d $(DESTDIR)$(MANDIR)/man1
	install -m 644 $(MANPAGE) $(DESTDIR)$(MANDIR)/man1/$(BIN_NAME).1

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(BIN_NAME)
	rm -f $(DESTDIR)$(MANDIR)/man1/$(BIN_NAME).1
