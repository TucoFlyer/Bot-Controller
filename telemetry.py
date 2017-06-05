#!/usr/bin/env python3

import socket
import sys
import time
import binascii
import struct

PORT = 9024
CONTROLLER = ('10.32.0.1', PORT)
FLYER = ('10.32.0.8', PORT)

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(CONTROLLER)

while True:
    pkt, srcaddr = s.recvfrom(4096)
    cmd = pkt[0]
    body = pkt[1:]

    if srcaddr == FLYER and cmd == 0x02:
        print("Flyer Sensors: ", struct.unpack('<' + 'i' * ((len(body)//4)), body))

    else:
        print(srcaddr, binascii.b2a_hex(pkt))

