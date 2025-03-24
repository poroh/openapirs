#
# Open API spec compiler
#

all:
	cargo build
	cargo test
	cargo clippy
