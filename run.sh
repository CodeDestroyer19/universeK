#!/bin/bash

qemu-system-i386 \
    -kernel kernel.bin \
    -serial stdio \
    -vga std \
    -m 128M \
    -no-reboot \
    -no-shutdown \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
    -display gtk \
    -usb \
    -device usb-mouse 