
# nx

> Userland library for Nintendo Switch homebrew (and other potential purposes), written in pure Rust and some assembly bits

> ### API docs are hosted [here](https://aarch64-switch-rs.github.io/nx/), and examples can be found [here](https://github.com/aarch64-switch-rs/examples)

> ### Setup guide to start developing Rust homebrew can be found [here](https://github.com/aarch64-switch-rs/setup-guide)

## TODO list

- Finish implementing all IPC/SF interfaces (+ their results):

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

  - Add other unimplemented services not mentioned above

- TIPC server support

- Extend hw-crypto support

- More FS interfaces:

  - PFS0

  - Romfs

  - NCA

- NRO Romfs support

- Finish implementing all SVCs

- Actual hw-rendering? (maybe as a separate lib like [deko3d](https://github.com/devkitPro/deko3d)?)

- Finish SMC support

- Finish waitable support

- Improve library applet support (specific implementations, etc.)

- Optimize IPC code to generate even better asm (like libnx or nnsdk)

- Finish documenting still-undocumented modules (`ipc`, `svc` and `service`)

- 32-bit support (see the corresponding branch)

- Console support

- Think about `std` support?

## Credits

- [libnx](https://github.com/switchbrew/libnx) and its contributors for being the base of this project.

- [Atmosphère](https://github.com/Atmosphere-NX/Atmosphere) and its contributors for being another base of this project.