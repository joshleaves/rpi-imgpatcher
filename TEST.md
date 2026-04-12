# Build test.img

Using the [fat32.img](https://github.com/rafalh/rust-fatfs/blob/master/resources/fat32.img) file from [rust-fatfs](https://github.com/rafalh/rust-fatfs).

```bash
$ wget https://raw.githubusercontent.com/rafalh/rust-fatfs/master/resources/fat32.img
$ dd if=/dev/zero of=test.img bs=1m count=40
$ hdiutil attach -imagekey diskimage-class=CRawDiskImage -nomount test.img
/dev/diskX

$ diskutil partitionDisk /dev/diskX MBR \
  FAT32 DUMMY 35M \
  ExFAT NOISE 2M

$ diskutil detach /dev/diskX
$ diskutil attach /dev/diskX

$ dd if=fat32.img of=/dev/disk26s1 bs=1m conv=sync
$ dd if=/dev/random of=/dev/disk26s2 bs=1m conv=sync
$ diskutil detach /dev/diskX
```