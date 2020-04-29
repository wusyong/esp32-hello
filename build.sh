#!/usr/bin/env bash

set -euo pipefail

chip="${1:-esp32}"

serial_port="$(ls /dev/tty.usbserial-* 2>/dev/null | head -n 1 || true)"

set -euo pipefail

target="xtensa-${chip}-none-elf"

cross build --release --target "${target}" -vv

if [[ -z "${serial_port}" ]]; then
  exit
fi

# esptool.py --chip "${chip}" --port "${serial_port}" --baud 115200 --before default_reset --after hard_reset erase_flash

bootloader_offset=0x0000

if [[ "${chip}" = 'esp32' ]]; then
  bootloader_offset=0x1000
fi

esptool.py --chip "${chip}" --port "${serial_port}" --baud 115200 --before default_reset --after hard_reset write_flash \
  -z --flash_mode dio \
  --flash_freq 80m \
  --flash_size detect \
  "${bootloader_offset}" "target/${target}/esp-build/bootloader/bootloader.bin" \
  0x8000 "target/${target}/esp-build/partitions.bin" \
  0x10000 "target/${target}/release/esp32-hello.bin"

python -m serial.tools.miniterm --rts=0 --dtr=0 "${serial_port}" 115200
