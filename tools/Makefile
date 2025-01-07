KERNEL_VERSION:=${KERNEL_VERSION}
ARCHITECTURE:=arm64
BUSYBOX_VERSION:=${BUSYBOX_VERSION}

kernel:
	mkdir -p ./obj/linux-${KERNEL_VERSION} && \
		cd linux-${KERNEL_VERSION} && \
		make ARCH=$(ARCHITECTURE) O=../obj/linux-${KERNEL_VERSION} defconfig && \
		make V=1 ARCH=$(ARCHITECTURE) O=../obj/linux-${KERNEL_VERSION} -j1 \
		> /app/stdout.log 2> /app/stderr.log
	mkdir -p /app/assets
	cp ./obj/linux-${KERNEL_VERSION}/arch/$(ARCHITECTURE)/boot/Image /app/assets/kernel-${KERNEL_VERSION}

.PHONY: initramfs
initramfs:
	mkdir -p ./initramfs/{bin,sbin,etc,proc,sys,usr/{bin,sbin}}
	cp ./configs/busybox-${BUSYBOX_VERSION}*.config ./busybox-${BUSYBOX_VERSION}/.config
	( \
	  cd ./busybox-${BUSYBOX_VERSION}/ ; \
	  make -j1 ; \
	  file busybox ; \
	  cp busybox ../initramfs/bin/ ; \
	)
	chmod +x ./initramfs/bin/busybox
	cp ./init.sh ./initramfs/init
	cd ./initramfs && \
		find . -print0 | cpio --null --create --verbose --format=newc | gzip -9 > ../obj/initramfs
	mkdir -p /app/assets
	cp ./obj/initramfs /app/assets

