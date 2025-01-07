# Test results comparison

## simple-vm

ubuntu@9f6edd2bd57d:/$ cat /proc/cpuinfo
processor : 0
BogoMIPS : 48.00
Features : fp asimd evtstrm aes pmull sha1 sha2 crc32 atomics fphp asimdhp cpuid asimdrdm jscvt fcma lrcpc dcpop sha3 asimddp sha512 asimdfhm dit uscat ilrcpc flagm sb paca pacg dcpodp flagm2 frint
CPU implementer : 0x61
CPU architecture: 8
CPU variant : 0x0
CPU part : 0x000
CPU revision : 0

ubuntu@9f6edd2bd57d:/$ openssl speed
Doing md5 for 3s on 16 size blocks: 17193045 md5's in 3.00s
Doing md5 for 3s on 64 size blocks: 10366539 md5's in 2.99s
Doing md5 for 3s on 256 size blocks: 4688920 md5's in 3.00s

ubuntu@9f6edd2bd57d:~$ uname -a
Linux 9f6edd2bd57d 6.1.26 #3 SMP PREEMPT Tue Jan 7 05:28:55 UTC 2025 aarch64 aarch64 aarch64 GNU/Linux

## qemu-system-aarch64

ubuntu@9f6edd2bd57d:~$ openssl speed
Doing md5 for 3s on 16 size blocks: 1644372 md5's in 2.97s
Doing md5 for 3s on 64 size blocks: 1464011 md5's in 2.97s
Doing md5 for 3s on 256 size blocks: 1097790 md5's in 2.97s
Doing md5 for 3s on 1024 size blocks: 563837 md5's in 2.97s

ubuntu@9f6edd2bd57d:~$ cat /proc/cpuinfo
processor : 0
BogoMIPS : 125.00
Features : fp asimd evtstrm aes pmull sha1 sha2 crc32 cpuid
CPU implementer : 0x41
CPU architecture: 8
CPU variant : 0x0
CPU part : 0xd08
CPU revision : 3

ubuntu@9f6edd2bd57d:~$ uname -a
Linux 9f6edd2bd57d 6.1.26 #3 SMP PREEMPT Tue Jan 7 05:28:55 UTC 2025 aarch64 aarch64 aarch64 GNU/Linux
