.PHONY: install-deps build-daemon-web build-firmware run-installer package-installer clean help

TAURI_ENV ?=

help:
	@echo "Usage: make <target>"
	@echo ""
	@echo "  install-deps       Install Rust ARM target and all npm dependencies"
	@echo "  build-daemon-web   Build the daemon's embedded web UI"
	@echo "  build-firmware     Cross-compile rootshell and daemon for the device (depends on build-daemon-web)"
	@echo "  run-installer      Launch the GUI installer in dev mode (builds prerequisites)"
	@echo "  package-installer  Bundle a distributable release of the GUI installer"
	@echo "  clean              Remove all build artifacts"
	@echo ""
	@echo "First time setup:"
	@echo "  make install-deps && make run-installer"

install-deps:
	rustup target add armv7-unknown-linux-musleabihf
	npm --prefix daemon/web install
	npm --prefix installer-gui install

build-daemon-web:
	npm --prefix daemon/web install
	npm --prefix daemon/web run build

build-firmware: build-daemon-web
	cargo build-rootshell-firmware-devel
	cargo build-daemon-firmware-devel

run-installer: build-firmware
	npm --prefix installer-gui install
	$(TAURI_ENV) npm --prefix installer-gui run tauri dev

package-installer: build-firmware
	npm --prefix installer-gui install
	npm --prefix installer-gui run tauri build

clean:
	cargo clean
	rm -rf daemon/web/build
