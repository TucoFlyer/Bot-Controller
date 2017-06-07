#!/usr/bin/env python3

import socket
import sys
import struct
import time
import math

PORT = 9024
FLYER = ('10.32.0.8', PORT)
MSG_LEDS = b'\x05'
MAX_LEDS = 200


def shader(frame):
    def pixel(n):
        blip = math.pow(math.sin(n / 30.0 + frame * 0.02) * 0.5 + 0.5, 40.0)
        tint = (0.8, 1.0, 0.2)

        bright = int(max(0, min(31, 0.5 + 31 * math.pow(blip, 8.0))))
        r = int(max(0, min(255, 0.5 + 0xff * blip * tint[0] + 0.5)))
        g = int(max(0, min(255, 0.5 + 0xff * blip * tint[1] + 0.5)))
        b = int(max(0, min(255, 0.5 + 0xff * blip * tint[2] + 0.5)))

        return 0xE0000000 | (bright << 24) | (r << 16) | (g << 8) | b
    return pixel

def main():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    frame = 0
    while True:
        pixels = list(map(shader(frame), range(MAX_LEDS)))
        s.sendto(MSG_LEDS + struct.pack('>' + MAX_LEDS * 'I', *pixels), FLYER)
        time.sleep(1/100.0)
        frame += 1


if __name__ == '__main__':
    main()
