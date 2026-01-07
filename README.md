# An arbitrarily large FUSE filesystem

This is a tool I built to test how programs behave when attempting to scan a large directory.
But I didn't want to actually create a filesystem containing thousands or millions of files,
so instead I built a FUSE filesystem that pretends to contain lots of files.

### Features

Several parameters are user-configurable:
- Number of directories
- Number of files per directory
- Throttling delay (to make the costs of deep scanning more obvious)

### Usage notes

Note: `tarpit` has only been tested on Linux.

To run the filesystem, first create a mount directory (e.g. `mnt`) and then
build the binary and run it. For example:
- `cargo build -r`
- `target/release/tarpit mnt`

Note that some IDEs may get unhappy if you mount a huge faux filesystem inside the
project directory.

### Verbose logging

To see every request that hits the filesystem, use:
- `RUST_LOG=debug tarpit <args>`

### Build notes

You must have the fuse header files installed.
On debian/ubuntu, `apt install libfuse3-dev` is sufficient.
