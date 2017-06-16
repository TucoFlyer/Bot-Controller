#!/usr/bin/env python3

import time
import argparse
import struct

from fyproto import Packet
from flyergimbal import GimbalPort
from fysocketserver import run_server_thread
from tinyjoy import deadzone, JoystickThread


def controller(gimbal, js, hz=75.0, yaw_limits=(450, 3800), pitch_limits=(-10000, 10000)):

    # Follow loops all off
    gimbal.setVectorParam(number=0x63, value=(0,0,0))

    # Zero the initial velocity that we'll be setting later
    gimbal.setVectorParam(number=0x03, value=(0,0,0))

    # Turn motors on if they aren't already
    gimbal.setMotors(True)

    while True:
        time.sleep(1.0/hz)
        controls = js.state()

        # Yaw is a speed (angle per time) integrated on MCU0
        command_yaw_speed = int(pow(deadzone(controls.get('rx', 0)), 3.0) * -300)

        # In this example the Pitch input is speed, but we are commanding
        # the gimbal by sending a joystick packet (in faux-servo units)
        # which applies an offset to the target of its follow loop
        command_pitch_speed = int(pow(deadzone(controls.get('ry', 0)), 3.0) * -150)

        # For this particular controller's purposes, our most appropriate
        # absolute notion of yaw (relative to the robot body) will be the
        # magnetic encoder on the yaw axis.
        current_yaw = gimbal.getParam(number=0x2c, target=0)

        # Current pitch vs the horizon comes from the gyro angle
        current_pitch = gimbal.getParam(number=0x09, target=2)

        # Not perfect, but put the brakes on if we're out of yaw range
        if current_yaw <= yaw_limits[0] and command_yaw_speed < 0:
            command_yaw_speed = 0
        if current_yaw >= yaw_limits[1] and command_yaw_speed > 0:
            command_yaw_speed = 0

        # Send latest yaw and pitch speeds
        gimbal.setParam(number=0x03, target=0, value=command_yaw_speed)
        gimbal.setParam(number=0x03, target=2, value=command_pitch_speed)

        # Status!
        print("Yaw: current=%d speed=%d  Pitch: current=%d speed=%d" % (
            current_yaw, command_yaw_speed,
            current_pitch, command_pitch_speed))


def main():
    parser = argparse.ArgumentParser(description='Simple remote for the Feiyu Tech gimbal')
    args = parser.parse_args()
    js = JoystickThread()
    gimbal = GimbalPort(verbose=False)
    run_server_thread(gimbal)
    controller(gimbal, js)


if __name__ == '__main__':
    main()
