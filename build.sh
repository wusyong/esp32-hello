#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "${0}")"

FLASH_BAUDRATE=460800
MONITOR_BAUDRATE=115200

serial_port="$(find /dev -name 'tty.usbserial-*' 2>/dev/null | head -n 1 || true)"

chip="${1:-esp32}"
target="xtensa-${chip}-none-elf"

profile=release

IDF_PATH="$(pwd)/esp-idf"
export IDF_PATH

IDF_TOOLS_PATH="$(pwd)/target/esp-idf-tools"
export IDF_TOOLS_PATH

mkdir -p "${IDF_TOOLS_PATH}"

cross build ${profile:+--${profile}} --target "${target}" --package app -vv

cross doc ${profile:+--${profile}} --target "${target}" --no-deps

if [[ -z "${serial_port}" ]]; then
  exit
fi

# esptool.py --chip "${chip}" --port "${serial_port}" --baud "${FLASH_BAUDRATE}" --before default_reset --after hard_reset erase_flash

bootloader_offset=0x0000

if [[ "${chip}" = 'esp32' ]]; then
  bootloader_offset=0x1000
fi

time esptool.py --chip "${chip}" --port "${serial_port}" --baud "${FLASH_BAUDRATE}" --before default_reset --after hard_reset write_flash \
  -z --flash_mode dio \
  --flash_freq 80m \
  --flash_size detect \
  "${bootloader_offset}" "target/${target}/esp-build/bootloader/bootloader.bin" \
  0x8000 "target/${target}/esp-build/partitions.bin" \
  0x10000 "target/${target}/${profile:-debug}/app.bin"

python -m serial.tools.miniterm --raw --exit-char=3 --rts=0 --dtr=0 "${serial_port}" "${MONITOR_BAUDRATE}"
