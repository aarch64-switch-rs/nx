
# nx

> Userland AARCH64 homebrew library for Nintendo Switch, written entirely in Rust

> ### API docs are hosted [here](https://aarch64-switch-rs.github.io/nx/), and examples can be found [here](https://github.com/aarch64-switch-rs/examples)

## TODO

- Finish implementing all IPC/SF interfaces (+ their results)

  - Finish applet services

  - Finish fatal services

  - Finish fs services

  - Finish hid services

  - Finish ldr services

  - Add lm:get

  - Finish mii services

  - Finish nv servuces

  - Support HTC/TMA?

  - Finish pm services

  - Finish psc services

  - Finish psm services

  - Finish settings services

  - Finish spl services

  - Finish usb services

  - Finish vi services

  - Add more unimplemented services

- TIPC server support

- Finish hw-crypto support: hw-accelerated AES, etc.

- More fs interfaces

  - PFS0

  - Romfs

  - NCA

- NRO Romfs support

- Finish implementing all SVCs

- Proper hw rendering in gpu? (maybe as a separate lib?)

- Rewrite `nx::input` module (proper types mostly)

- Finish SMC support

- Finish waitable support

- Improve library applet support (proper implementations, etc.)

- Optimize IPC code to generate better asm (like libnx or nnsdk)

- 32-bit support

## Credits

- [libnx](https://github.com/switchbrew/libnx) and its contributors for being the base of this project.

- [Atmosphère](https://github.com/Atmosphere-NX/Atmosphere) and its contributors for being another base of this project.