# moulinette

## What have been done ?

- [X] First step: restrict environment with Cgroup
- [X] Second step: remove capabilities
- [ ] Bonus: usable by a normal user
- [X] Third step: poor isolation with chroot
- [X] Fourth step: filter syscalls with seccomp
- [X] Fifth step: automate the creation of the env by pulling a docker image
- [X] Sixth step: real isolation by creating new namespaces
- [ ] Bonus: Usable by a normal user
- [X] Seventh step: avoid leaks of information (done with pivot root and unmounting old root)
- [X] Seventh bis: mount the student code
- [X] Eighth step: Container with unique hostname
- [X] Ninth step: pivot root
- [ ] Tenth step: Sandbox connected to network interfaces

## How to run

### Using cargo

```sh
cargo build --release
sudo RUST_LOG=info target/release/moulinette -I library/alpine:latest /bin/sh # Run with logs
sudo target/release/moulinette -I library/alpine:latest /bin/sh # Run without logs
```

### Using the makefile

```sh
make release # Build in release mode
make debug # Build in debug mode
make run-release # Run alpine:latest in release mode
make debug-release # Run alpine:latest in debug mode
```