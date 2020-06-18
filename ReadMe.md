# Dependencies

- [Docker](https://docs.docker.com/get-docker/)
- `jq`
- [`cross`](https://github.com/rust-embedded/cross) with images
  from https://github.com/reitermarkus/cross/tree/xtensa:
   ```
  git clone -b xtensa https://github.com/reitermarkus/cross
  cd cross
  cargo install --path . --force
  ./build-docker-image.sh xtensa-esp32-none-elf
  ```
- [`esptool.py`](https://github.com/espressif/esptool)

# Building

When building the first time, fetch the submodules using

```
git submodule update --init --recursive
```

Afterwards, you can build the project using

```
./build.sh [--chip <chip> (default: esp32)] [--release] [--package <package> (default: app)] [--example <example>] [--flash-baudrate <baud> (default: 460800)] [--erase-flash]
```
