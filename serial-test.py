#!/usr/bin/env python3

import socket
import sys
import time
import binascii

PORT = 9024
CONTROLLER = ('10.32.0.1', PORT)
FLYER = ('10.32.0.8', PORT)

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(CONTROLLER)

while True:
    s.sendto(binascii.a2b_hex(sys.argv[2]), FLYER)
    time.sleep(float(sys.argv[1]))

