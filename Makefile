debug:
	cargo build -vvvvv --target=armv7-unknown-linux-gnueabihf
	#arm-bela-linux-gnueabihf-strip target/armv7-unknown-linux-gnueabihf/debug/mbms
	scp target/armv7-unknown-linux-gnueabihf/debug/mbms root@bela.local:~

release:
	cargo build -vvvvv --target=armv7-unknown-linux-gnueabihf --release
	# arm-bela-linux-gnueabihf-strip target/armv7-unknown-linux-gnueabihf/release/mbms
	scp target/armv7-unknown-linux-gnueabihf/release/mbms root@bela.local:~

all: release
