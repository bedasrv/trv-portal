.PHONY: build build-all release clean install strip

TARGET := aarch64-unknown-linux-musl
RELEASE := target/$(TARGET)/release
BINS := trv-portal-detect trv-portal-mode trv-portal-clone trv-portal-cgi

build:
	cross build --release --target $(TARGET)

build-all: build
	@ls -lh $(RELEASE)/trv-portal-*

strip: build
	aarch64-linux-gnu-strip $(RELEASE)/trv-portal-*

stage: strip
	mkdir -p files/usr/bin
	cp $(RELEASE)/trv-portal-* files/usr/bin/

clean:
	cargo clean
	rm -rf files/usr/bin/trv-portal-*

install: stage
	scp files/usr/bin/trv-portal-* root@192.168.1.1:/usr/bin/
	ssh root@192.168.1.1 'chmod +x /usr/bin/trv-portal-*'
