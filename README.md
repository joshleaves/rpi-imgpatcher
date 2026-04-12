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

---

## Features (planned / in progress)

- Extract boot partition from `.img`
- Modify FAT filesystem safely
- Inject files before first boot
- Deterministic image patching
- CLI interface
- Optional declarative patch format (à la Dockerfile)

---

## Non-goals

- Modifying the root filesystem (ext4)
- Rebuilding full OS images
- Replacing tools like Yocto / Buildroot

Instead, the focus is on:
> First-boot provisioning via boot partition patching

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
