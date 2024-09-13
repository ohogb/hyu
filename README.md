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

- `super + esc` exits hyu
- `super + t` spawns `foot` terminal
- `super + c` closes active window
- `super + j` focuses the next window in the stack
- `super + k` focuses the previous window in the stack
- `super + shift + j` move current window down the stack
- `super + shift + k` move current window up the stack
