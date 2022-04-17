
# nx

Userland homebrew library for Nintendo Switch written entirely in Rust

> ### API docs are hosted [here](https://aarch64-switch-rs.github.io/nx/), and examples can be found [here](https://github.com/aarch64-switch-rs/examples)

## TODO

- Finish implementing all ipc/sf interfaces (+ their results)

- TIPC server support

- Crypto: hw-accelerated AES, etc.

- More fs interfaces (PFS0, Romfs, NCA filesystem support...)

- NRO Romfs support

- Finish implementing all SVCs

- Proper hw rendering in gpu? (maybe as a separate lib?)

- (...)

## Credits

- [libnx](https://github.com/switchbrew/libnx) and its contributors for being the base of this project.

- [Atmosphere](https://github.com/Atmosphere-NX/Atmosphere) and its contributors for being another base of this project.