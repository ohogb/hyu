# hyu

A wayland compositor written from scratch

![preview](./assets/preview.png)
> Programs running inside hyu

### Requirements

- nightly rust
- newish linux kernel
- `libxkbcommon`
- `libEGL`
- `libgbm`
- `libudev`
- `libinput`

### Running in tty

```sh
cargo run
```

#### Keybinds

- `super + t` spawns `foot` terminal
- `super + c` closes active window
