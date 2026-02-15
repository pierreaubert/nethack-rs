.PHONY: build link clean

TARGET_DIR = target/riscv64emac-unknown-none-polkavm/release
ELF = $(TARGET_DIR)/nh_polkadot
OUTPUT = nh_polkadot.polkavm

build:
	cargo build --release

link: build
	polkatool link --strip --output $(OUTPUT) $(ELF)

clean:
	cargo clean
	rm -f $(OUTPUT)
