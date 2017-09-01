#!/usr/bin/env python3

import socket
import sys
import struct
import time
import math

PORT = 9024
FLYER = ('10.32.0.10', PORT)
MSG_LEDS = b'\x05'
MAX_LEDS = 200


def shader(frame):

    blips = [
        (frame * 0.02,  (0.8, 1.0, 0.2)),
        (frame * 0.03,  (0.2, 0.6, 0.9)),
        (-frame * 0.01, (0.9, 0.3, 0.2)),
    ]

    def pixel(n):

        br = [ math.pow(math.sin(n / 10.0 - center), 40.0) for center, color in blips ]

        rgb = [
            sum( br[i] * tint[ch] for i, (center, tint) in enumerate(blips) )
            for ch in range(3)   
        ]

        bright = int(max(0, min(31, 1.0 + 4.0 * math.pow(sum(rgb), 4.5))))
        r = int(max(0, min(255, 0.5 + 0xff * rgb[0])))
        g = int(max(0, min(255, 0.5 + 0xff * rgb[1])))
        b = int(max(0, min(255, 0.5 + 0xff * rgb[2])))

        return 0xE0000000 | (bright << 24) | (r << 16) | (g << 8) | b
    return pixel

def main():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    frame = 0
    last_t = 0
    while True:
        if (frame & 0xf) == 0:
            t = time.time()
            dt = t - last_t
            last_t = t
            print("fps ~= ", 16.0 / dt)

        pixels = list(map(shader(frame), range(MAX_LEDS)))
        s.sendto(MSG_LEDS + struct.pack('>' + MAX_LEDS * 'I', *pixels), FLYER)
        time.sleep(1/100.0)
        frame += 1


if __name__ == '__main__':
    main()
