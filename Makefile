DISTRO:=ubuntu-24.04
KERNEL_VERSION:=6.1.26

.PHONY: build-kernel
build-kernel:
	@docker volume create kernel-build-cache
	@docker build -f tools/Dockerfile -t kernel-builder .
	@docker run -it --mount source=kernel-build-cache,target=/builder/obj -v $(shell pwd):/app kernel-builder

.PHONY: build-initramfs
build-initramfs:
	@docker volume create kernel-build-cache
	@docker build -f tools/Dockerfile -t kernel-builder .
	@docker run -it --mount source=kernel-build-cache,target=/builder/obj -v $(shell pwd):/app kernel-builder initramfs

.PHONY: build-rootfs
build-rootfs:
	@docker volume create kernel-build-cache
	@docker build -f tools/rootfs/Dockerfile.${DISTRO} -t rootfs-${DISTRO} .
	@docker run -it --mount source=kernel-build-cache,target=/builder/obj -v $(shell pwd):/app --privileged --cap-add=CAP_MKNOD rootfs-${DISTRO}

.PHONY: build-shell
build-shell:
	@docker run -it --mount source=kernel-build-cache,target=/builder/obj -v $(shell pwd):/app --entrypoint bash kernel-builder

.PHONY: debug
debug:
	cargo update -p block2 --precise 0.2.0-alpha.8
	cargo update -p objc2 --precise 0.3.0-beta.5
	cargo build --bin simple-vm
	codesign -f --entitlement ./virt-fwk.entitlements -s - target/debug/simple-vm

.PHONY: simple-vm
simple-vm: debug
	#@RUST_BACKTRACE=full ./target/debug/simple-vm --kernel $(shell pwd)/assets/kernel-${KERNEL_VERSION} --initrd $(shell pwd)/assets/initramfs --disk $(shell pwd)/assets/${DISTRO}.img
	RUST_BACKTRACE=full ./target/debug/simple-vm --kernel $(shell pwd)/assets/kernel-${KERNEL_VERSION} --disk $(shell pwd)/assets/${DISTRO}.img

.PHONY: check
check:
	cargo clippy

.PHONY: publish
publish:
	cargo publish -p virt-fwk --dry-run

.PHONY: docs
docs:
	cargo doc -p virt-fwk
	@cd ./target/doc && python -m http.server
