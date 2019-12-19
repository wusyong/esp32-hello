#!/usr/bin/env bash

set -euo pipefail

serial_port=

if [ -e /dev/tty.usbserial-1420 ]; then
  serial_port=/dev/tty.usbserial-1420
elif [ -e /dev/cu.SLAB_USBtoUART ]; then
  serial_port=/dev/cu.SLAB_USBtoUART
fi

set -euo pipefail

cross build --release --target xtensa-esp32-none-elf

if [[ -z $serial_port ]]; then
  exit
fi

esptool.py --chip esp32 --port "${serial_port}" --baud 115200 --before default_reset --after hard_reset write_flash \
  -z --flash_mode dio \
  --flash_freq 80m \
  --flash_size detect \
  0x1000 target/esp-build/bootloader/bootloader.bin \
  0x8000 target/esp-build/partitions.bin \
  0x10000 target/xtensa-esp32-none-elf/release/esp32-hello.bin

python -m serial.tools.miniterm --rts=0 --dtr=0 "${serial_port}" 115200
