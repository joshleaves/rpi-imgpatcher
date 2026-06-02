# Patcherfile

A `Patcherfile` is a small, line-based recipe format used by `rpi-imgpatcher` to patch Raspberry Pi images in a predictable order.

The format is intentionally minimal. It is not a build system, not a shell replacement, and not a general-purpose DSL.

## Execution model

A `Patcherfile` is executed from top to bottom.

Each instruction is processed in order.

A typical flow looks like this:

```text
FROM "base.img"
EXEC cp firstrun.template.sh firstrun-client-a.sh
ADD FILE "boot/firstrun.sh" "firstrun-client-a.sh"
SAVE "client-a.img"
```

In practice:

- `FROM` opens the source image and starts a patch session.
- `SHELL` runs a host shell command.
- `EXEC` runs a host program directly with explicit arguments.
- `ADD FILE` copies a host file into the FAT partition.
- `APPEND FILE` appends a host file to an existing FAT file.
- `SAVE` writes a new output image.

## Syntax rules

Each instruction must be written on a single line.

Arguments containing spaces must be quoted with double quotes.

Blank lines are ignored.

Instruction names are uppercase.

Environment variables of the form `$VAR` are expanded by the `Patcherfile` parser before instructions are processed. Variables are resolved using the host environment. Undefined variables are replaced with an empty string.

### Environment variables

Variables can be used inside arguments using the `$VAR` syntax.

```text
FROM "base.img"
SHELL cat firstrun.sh | sed "s|spark-noconf|$NAME|g" > firstrun-$NAME.sh
ADD FILE "boot/firstrun-$CLIENT.sh" "./firstrun-$CLIENT.sh"
SAVE "image-$CLIENT.img"
```

Notes:

- Variable expansion is purely textual and happens before parsing.
- Only simple `$VAR` patterns are supported.
- `${VAR}` syntax is not supported.
- Escaping (`\$VAR`) is not supported.
- Undefined variables are replaced with an empty string.
- Variable expansion does not apply to instruction names.

## Instructions

## `FROM`

Starts a patch session from a source image.

```text
FROM "tests/fixtures/test.img"
```

Arguments:

- `source_image`: host path to the source image

Rules:

- A valid `Patcherfile` must contain exactly one `FROM` before any operation that touches the image.
- `ADD FILE`, `APPEND FILE`, and `SAVE` require a prior `FROM`.
- A valid `Patcherfile` must contain exactly one `SAVE`, and `SAVE` must be the last instruction.

## `SHELL`

Runs a shell command on the host system.

```text
SHELL cp firstrun.template.sh firstrun-client-a.sh
SHELL cat firstrun.sh | sed "s|spark-noconf|client-a|g" > firstrun-client-a.sh
```

Arguments:

- `command`: shell command string

Notes:

- `SHELL` runs on the host, not inside the image.
- It executes through `/bin/sh -o pipefail -c`.
- Use it when you need shell features (pipes, redirects, substitutions, chaining).
- If the command exits with a non-zero status, execution stops.

## `EXEC`

Runs a host program directly (no shell parsing).

```text
EXEC cp firstrun.template.sh firstrun-client-a.sh
EXEC chmod +x firstrun-client-a.sh
```

Arguments:

- first token: program path/name
- following tokens: arguments passed as-is

Notes:

- `EXEC` runs on the host, not inside the image.
- No shell expansion is performed (`|`, `>`, `&&`, `$(...)` are not interpreted).
- Use `SHELL` if you need shell features.
- If the command exits with a non-zero status, execution stops.

## `ADD FILE`

Copies a host file into the FAT partition of the image.

```text
ADD FILE "boot/firstrun.sh" "./firstrun-client-a.sh"
```

Arguments:

- `fat_path`: destination path inside the image FAT partition
- `host_file`: source file on the host filesystem

Behavior:

- Creates or replaces the destination file.
- Parent directories inside the FAT image are created automatically if needed.

Convention:

- Destination is always written first.

## `APPEND FILE`

Appends the contents of a host file to a file inside the FAT partition.

```text
APPEND FILE "boot/cmdline.txt" "./cmdline-extra.txt"
```

Arguments:

- `fat_path`: destination path inside the image FAT partition
- `host_file`: source file on the host filesystem

Behavior:

- Appends to the destination file if it exists.
- Creates the destination file if it does not exist.
- Parent directories inside the FAT image are created automatically if needed.

Convention:

- Destination is always written first.

## `APPEND CMDLINE`

Appends a configuration string to the `cmdline.txt` boot file.

```text
APPEND CMDLINE "systemd.run=/boot/firstrun.sh systemd.run_success_action=reboot systemd.unit=kernel-command-line.target"
```

Arguments:

- `append_conf`: Configuration string.

Behavior:

- Operates only on `cmdline.txt`.
- The existing content is treated as a single line.
- Trailing white spaces are removed.
- A single space is inserted if needed.
- The append_conf string is appended to the end of the line.
- The file is written back without a trailing newline.

Use case:

- Typically used to configure `firstrun.sh` execution on first boot.

## `SAVE`

Writes the patched image to a new output file.

```text
SAVE "./client-a.img"
```

Arguments:

- `output_image`: host path for the generated image

Behavior:

- The original source image remains untouched.
- `SAVE` finalizes the current patch session.

## Valid example

```text
FROM "tests/fixtures/test.img"
SHELL printf '%s\n' 'first boot script' > ./firstrun-client-a.sh
ADD FILE "boot/firstrun.sh" "./firstrun-client-a.sh"
APPEND FILE "boot/cmdline.txt" "./cmdline-extra.txt"
SAVE "./client-a.img"
```

## Common errors

## Missing `FROM`

This is invalid:

```text
ADD FILE "boot/firstrun.sh" "./firstrun.sh"
SAVE "./out.img"
```

Because no source image was opened first.

## Missing output instruction

This is invalid:

```text
FROM "tests/fixtures/test.img"
ADD FILE "boot/firstrun.sh" "./firstrun.sh"
```

Because the patch session never ends with `SAVE`.

## Notes

`Patcherfile` is intentionally small.

If you need host-side preparation, use `EXEC` for direct argv-style commands and `SHELL` for shell composition (pipes/redirections), then use `ADD FILE` or `APPEND FILE`.

The goal is to keep image patching predictable, readable, and easy to audit.
