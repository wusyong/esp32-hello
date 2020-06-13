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

```
./build.sh
```
