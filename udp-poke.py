#!/usr/bin/env python3

import socket
import sys
import time
import binascii

PORT = 9024

delay = float(sys.argv[1])
dest = (sys.argv[2], PORT)
packet = binascii.a2b_hex(sys.argv[3])

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

while True:
    s.sendto(packet, dest)
    time.sleep(delay)

