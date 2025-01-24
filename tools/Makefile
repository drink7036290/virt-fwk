SHELL := /bin/bash

KERNEL_VERSION:=${KERNEL_VERSION}
ARCH:=arm64
BUSYBOX_VERSION:=${BUSYBOX_VERSION}
KERNEL_BUILD_PATH:=./obj/linux-${KERNEL_VERSION}

kernel:
	mkdir -p ${KERNEL_BUILD_PATH} && \
		cd linux-${KERNEL_VERSION} && \
		make ARCH=${ARCH} O=../${KERNEL_BUILD_PATH} defconfig && \
		make ARCH=${ARCH} O=../${KERNEL_BUILD_PATH} -j1 \
		> /app/stdout.log 2> /app/stderr.log
	mkdir -p /app/assets
	cp ${KERNEL_BUILD_PATH}/arch/${ARCH}/boot/Image /app/assets/kernel-${KERNEL_VERSION}

.PHONY: initramfs
initramfs:
	mkdir -p ./initramfs/{bin,sbin,etc,proc,sys,usr/{bin,sbin}}
	cp ./configs/busybox-${BUSYBOX_VERSION}-${ARCH}.config ./busybox-${BUSYBOX_VERSION}/.config
	( \
	  cd ./busybox-${BUSYBOX_VERSION}/ ; \
	  make -j1 ; \
	  file busybox ; \
	  chmod +x busybox ; \
	  mv busybox ../initramfs/bin/ ; \
	)
	cp ./init.sh ./initramfs/init
	cd ./initramfs && \
		find . -print0 | cpio --null --create --verbose --format=newc | gzip -9 > ../obj/initramfs
	mkdir -p /app/assets
	cp ./obj/initramfs /app/assets

