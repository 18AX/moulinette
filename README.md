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
- [ ] Seventh bis: mount the student code
- [X] Eighth step: Container with unique hostname
- [X] Ninth step: pivot root
- [ ] Tenth step: Sandbox connected to network interfaces