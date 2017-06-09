#!/usr/bin/env python3

import socket
import sys
import time
import binascii
import struct
from tinyjoy import deadzone, JoystickThread

PORT = 9024
WINCH = ('10.32.0.10', PORT)
MSG_WINCH_COMMAND = b'\x04'

js = JoystickThread()
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

while True:
    time.sleep(1.0/100.0)
    controls = js.state()

    velocity_target = int(pow(deadzone(controls.get('ry', 0)), 3.0) * 500)
    accel_max = 100
    force_min = 10
    force_max = 100000

    packet = MSG_WINCH_COMMAND + struct.pack('<iIii', velocity_target, accel_max, force_min, force_max)
    print('TX: %r' % binascii.b2a_hex(packet))
    s.sendto(packet, WINCH)
