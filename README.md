# rpi-imgpatcher

A lightweight tool to customize Raspberry Pi OS images by patching the boot (FAT) partition — without mounting the image or rebuilding the system.

---

## Overview

`rpi-imgpatcher` allows you to modify Raspberry Pi OS `.img` files in a simple, deterministic way.

Instead of:
- mounting disk images
- using loop devices
- rebuilding full images with tools like Yocto or Buildroot

This tool focuses on a much simpler workflow:

> Extract → Patch → Reassemble → Flash

It operates directly on the boot partition (FAT), enabling you to inject files and configuration that will be applied on first boot.

---

## Why?

Provisioning Raspberry Pi systems at scale is surprisingly painful.

Typical approaches involve:
- fragile shell scripts
- manual file copies
- post-flash SSH setup
- complex image build systems

`rpi-imgpatcher` aims to provide a clean middle ground:
- no full rebuilds
- no manual mounting
- reproducible image customization

--
## Build features
- `ffi`: Even if it was my first motivation, the base library doesn't requires it. Therefore, it becomes a feature.
- `ffi_debug`: Exposes a `char * rpi_imgpatcher_last_error_message()` function that returns a heap-allocated string containing the last error message encountered by the FFI (global, not thread-safe). The returned buffer must be freed manually using `void rpi_imgpatcher_last_error_free(const char *error)`.

- `buffered_copy`: While `std::io::copy` is already very efficient, a small (~5%) performance improvement can be observed with manual buffering in some I/O-heavy scenarios.  The default buffer size is 4MB (based on local benchmarks), but it can be overridden at build time using the `RPI_COPY_BUFFER_SIZE` environment variable.

---

## XZ support

`rpi-imgpatcher` supports `.xz` archives as inputs.

In practice, using an `.xz` file as the output format is usually a poor choice. Writing compressed images is significantly slower than writing plain `.img` files, and the resulting archive must still be decompressed before flashing, so the option was removed.

As for performance, here is a simple benchmark I ran using the same input image, once with a plain `.img` output and once with an `.img.xz` output:

```bash
$ time NAME=testred cargo run --release
    Finished release profile [optimized] target(s) in 0.11s
     Running target/release/rpi-imgpatcher

________________________________________________________
Executed in    5.79 secs    fish           external
   usr time    0.21 secs    0.36 millis    0.21 secs
   sys time    3.74 secs    1.22 millis    3.74 secs

$ time NAME=testredxz cargo run --release
    Finished release profile [optimized] target(s) in 0.12s
     Running target/release/rpi-imgpatcher

________________________________________________________
Executed in   58.12 secs    fish           external
   usr time   49.67 secs    0.43 millis   49.67 secs
   sys time    4.34 secs    1.91 millis    4.34 secs
```

For most workflows, .xz is best used as an input format or as a storage/archive format.

--

## Non-goals

- Modifying the root filesystem (ext4)
- Rebuilding full OS images
- Replacing tools like Yocto / Buildroot

Instead, the focus is on:
> First-boot provisioning via boot partition patching

---

## Huge thanks to:

- [mbrman](rust-disk-partition-management/mbrman)
- [rust-fatfs](https://github.com/rafalh/rust-fatfs)
- [lzma-rust2](https://github.com/hasenbanck/lzma-rust2)

---

## Current Status

🚧 Early development

This project is being built incrementally using:
- GitHub issues for each step
- small, testable milestones
- a core-first approach (Rust)

The API and CLI are not stable yet.

---

## Roadmap (high level)

- [x] Core image parsing
- [x] FAT extraction and modification
- [x] File injection
- [x] CLI tool
- [x] Validation and safety checks
- [x] Declarative patch format ([Patcherfile](./PATCHERFILE.md))
- [x] FFI bindings
- [x] Support for .xz archives

---

## Philosophy

Keep it simple.

- Prefer patching over rebuilding
- Prefer deterministic workflows over scripts
- Avoid unnecessary complexity

---

## Disclaimer

This project is not affiliated with the Raspberry Pi Foundation.

---

## License

MIT
