# Try building .config manually if no .config available

make defconfig ;
make menuconfig ;

## Test image and initramfs by running qemu

qemu-system-aarch64 \
 -machine virt,virtualization=true \
 -cpu cortex-a72 \
 -m 2048 \
 -nographic \
 -kernel "$(pwd)/assets/kernel-6.1.26" \
  -append "console=ttyAMA0 root=/dev/vda" \
  -initrd "$(pwd)/assets/initramfs" \
 -drive if=virtio,format=raw,file="$(pwd)/assets/ubuntu-24.04.img"

## kill this qemu by either of the following

ps aux \
 | rg "qemu-system-aarch64" \
 | awk '{print $2}' \
 | xargs kill -9

pkill -9 qemu-system-aarch64
