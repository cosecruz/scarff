# Makefile — developer shortcuts for building and testing Scarff cross-platform.
#
# Prerequisites:
#   cargo install cross   — for Linux/ARM cross-compilation
#   Docker running        — required by cross
#
# Usage:
#   make test             — run tests on native target
#   make build-all        — build for all release targets
#   make build-musl       — build static Linux binaries
#   make check            — fmt + clippy + check
#   make release          — local release build (native)
#   make dist             — build all targets and package to dist/

BINARY   := scarff
VERSION  := $(shell cargo metadata --no-deps --format-version 1 | \
              python3 -c "import sys,json; \
              pkgs = json.load(sys.stdin)['packages']; \
              print(next(p['version'] for p in pkgs if p['name']=='scarff'))")

# Native target detection
UNAME_S  := $(shell uname -s)
UNAME_M  := $(shell uname -m)

ifeq ($(UNAME_S),Linux)
  ifeq ($(UNAME_M),x86_64)
    NATIVE_TARGET := x86_64-unknown-linux-gnu
  else ifeq ($(UNAME_M),aarch64)
    NATIVE_TARGET := aarch64-unknown-linux-gnu
  endif
else ifeq ($(UNAME_S),Darwin)
  ifeq ($(UNAME_M),x86_64)
    NATIVE_TARGET := x86_64-apple-darwin
  else ifeq ($(UNAME_M),arm64)
    NATIVE_TARGET := aarch64-apple-darwin
  endif
endif

# ── Quality ───────────────────────────────────────────────────────────────────

.PHONY: check
check: fmt clippy build-check

.PHONY: fmt
fmt:
	cargo fmt --all -- --check

.PHONY: fmt-fix
fmt-fix:
	cargo fmt --all

.PHONY: clippy
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: build-check
build-check:
	cargo check --workspace --all-targets --all-features

.PHONY: test
test:
	cargo test --workspace --all-features --all-targets

.PHONY: test-verbose
test-verbose:
	cargo test --workspace --all-features --all-targets -- --nocapture

.PHONY: coverage
coverage:
	cargo llvm-cov --workspace --all-features --html
	@echo "Coverage report: target/llvm-cov/html/index.html"

.PHONY: deny
deny:
	cargo deny check

.PHONY: audit
audit: deny
	cargo audit

# ── Native build ──────────────────────────────────────────────────────────────

.PHONY: build
build:
	cargo build --release --bin $(BINARY)

.PHONY: run
run:
	cargo run --bin $(BINARY) --

# ── Cross-compilation ─────────────────────────────────────────────────────────

.PHONY: build-linux-x86-musl
build-linux-x86-musl:
	cross build --release --target x86_64-unknown-linux-musl --bin $(BINARY)

.PHONY: build-linux-arm64-musl
build-linux-arm64-musl:
	cross build --release --target aarch64-unknown-linux-musl --bin $(BINARY)

.PHONY: build-linux-arm64-gnu
build-linux-arm64-gnu:
	cross build --release --target aarch64-unknown-linux-gnu --bin $(BINARY)

.PHONY: build-musl
build-musl: build-linux-x86-musl build-linux-arm64-musl
	@echo "musl binaries built."

.PHONY: build-all
build-all:
	@echo "Building for all release targets..."
	@$(MAKE) build-linux-x86-musl
	@$(MAKE) build-linux-arm64-musl
	@$(MAKE) build-linux-arm64-gnu
	@echo "Cross-compilation complete."
	@echo "Native macOS and Windows targets require their respective runners."

# ── Packaging ─────────────────────────────────────────────────────────────────
# Creates archives in dist/ matching the release workflow naming convention.

DIST := dist

.PHONY: dist
dist: build-musl build
	@mkdir -p $(DIST)
	$(MAKE) _package TARGET=x86_64-unknown-linux-musl   EXT=tar.gz
	$(MAKE) _package TARGET=aarch64-unknown-linux-musl  EXT=tar.gz
	$(MAKE) _package TARGET=$(NATIVE_TARGET)             EXT=tar.gz
	@echo "Artefacts in $(DIST)/"
	@ls -lh $(DIST)/

.PHONY: _package
_package:
	@mkdir -p $(DIST)
	@ARCHIVE="$(BINARY)-$(VERSION)-$(TARGET).$(EXT)"; \
	cp "target/$(TARGET)/release/$(BINARY)" /tmp/$(BINARY); \
	tar czf "$(DIST)/$$ARCHIVE" -C /tmp "$(BINARY)" README.md; \
	sha256sum "$(DIST)/$$ARCHIVE" > "$(DIST)/$$ARCHIVE.sha256"; \
	rm /tmp/$(BINARY); \
	echo "Packaged: $(DIST)/$$ARCHIVE"

# ── Development utilities ─────────────────────────────────────────────────────

.PHONY: doc
doc:
	cargo doc --workspace --all-features --no-deps --open

.PHONY: doc-private
doc-private:
	cargo doc --workspace --all-features --no-deps --document-private-items --open

.PHONY: clean
clean:
	cargo clean
	rm -rf $(DIST)

.PHONY: update
update:
	cargo update

.PHONY: outdated
outdated:
	cargo outdated

.PHONY: install-tools
install-tools:
	cargo install cargo-deny
	cargo install cargo-llvm-cov
	cargo install cargo-audit
	cargo install cargo-outdated
	cargo install cross --git https://github.com/cross-rs/cross
	rustup component add llvm-tools-preview

.PHONY: help
help:
	@echo "Scarff build targets:"
	@echo ""
	@echo "  Quality:"
	@echo "    check           fmt + clippy + build-check"
	@echo "    fmt             check formatting (no changes)"
	@echo "    fmt-fix         apply formatting fixes"
	@echo "    clippy          lint with -D warnings"
	@echo "    test            run full test suite"
	@echo "    coverage        generate HTML coverage report"
	@echo "    deny            check licenses + advisories"
	@echo ""
	@echo "  Build:"
	@echo "    build           release build (native)"
	@echo "    build-musl      x86_64 + aarch64 musl static binaries"
	@echo "    build-all       all cross-compiled targets"
	@echo ""
	@echo "  Release:"
	@echo "    dist            build + package to dist/"
	@echo ""
	@echo "  Misc:"
	@echo "    install-tools   install all dev tools"
	@echo "    doc             open rustdoc in browser"
	@echo "    clean           cargo clean + dist/"
