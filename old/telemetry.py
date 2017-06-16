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


def unpack(b, fieldstr):
    fields = [f.rsplit('.', 1) for f in fieldstr.split()]
    names = (f[0] for f in fields)
    fmt = '<' + ''.join(f[1] for f in fields)
    vals = struct.unpack(fmt, b[:struct.calcsize(fmt)])
    return dict(zip(names, vals))

def clear():
    print("\x1b[2J")


clear()
while True:
    pkt, srcaddr = s.recvfrom(4096)
    cmd = pkt[0]
    body = pkt[1:]

    if srcaddr == FLYER and cmd == 0x02:
        sensors = unpack(body, '''
            xband_edges.I xband_average.I xband_ctr.I
            lidar0_range.I lidar1_range.I lidar2_range.I lidar3_range.I
            lidar0_ctr.I lidar1_ctr.I lidar2_ctr.I lidar3_ctr.I
            ir0.I ir1.I ir2.I ir3.I ir4.I ir5.I ir6.I ir7.I ir_ctr.I
            accel_x.h accel_y.h accel_z.h
            mag_x.h mag_y.h mag_z.h
            gyro_x.h gyro_y.h gyro_z.h
            euler_x.h euler_y.h euler_z.h
            quat_w.h quat_x.h quat_y.h quat_z.h
            linacc_x.h linacc_y.h linacc_z.h
            gravity_x.h gravity_y.h gravity_z.h
            imu_temp.B imu_calib_stat.B
            imu_reserved0.h imu_ctr.I
        ''')
        print("\x1b[1;1HFlyer Sensors:\n" + "\n".join('\t%-20s=%10d' % i for i in sorted(sensors.items())))

    else:
        print(srcaddr, binascii.b2a_hex(pkt))

