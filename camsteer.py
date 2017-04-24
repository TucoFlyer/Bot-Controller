#!/usr/bin/env python
#
# Work in progress for TucoCam steering!
# Xbox 360 controller -> Camera Gimbal
#

from __future__ import division
import evdev
import threading
import Adafruit_PCA9685
import time
import math


class JoystickThread(threading.Thread):
    def __init__(self, device=None):
        threading.Thread.__init__(self)
        self.device = device or self._default_joystick()
        self.axes = {}
        self._pending = {}
        self.setDaemon(True)
        for axis, info in self.device.capabilities().get(evdev.ecodes.EV_ABS, []):
            self.axes[axis] = (info, [None])

    def _default_joystick(self):
        """Return the first (sorted) device with an absolute X axis."""
        for fn in sorted(evdev.list_devices()):
            device = evdev.InputDevice(fn)
            for axis, info in device.capabilities().get(evdev.ecodes.EV_ABS, []):
                if axis == evdev.ecodes.ABS_X:
                    return device
        raise IOError('No joystick device found')

    def run(self):
        for event in self.device.read_loop():
           evc = evdev.categorize(event)
           if isinstance(evc, evdev.AbsEvent):
               self._pending[event.code] = event.value
           elif isinstance(evc, evdev.KeyEvent):
               self.onKey(evc)
           elif isinstance(evc, evdev.SynEvent):
               for axis, value in self._pending.items():
                   self.axes[axis][1][0] = value
               self._pendingValues = {}

    def onKey(self, event):
        print(event)

    def state(self):
        s = {}
        for axis, (info, box) in self.axes.items():
            if box[0] is not None:
                mapped = (box[0] - info.min) / (info.max - info.min)
                s[evdev.ecodes.ABS[axis].lower().split('_')[1]] = mapped
        return s

def set_servo_pulse(channel, usec):
    pwm.set_pwm(channel, 0, usec // (1000000 // 60 // 4096))

def motion_loop(jt, pwm):
    while True:
        state = jt.state()
        # Room for some motion control or other state machine here...
        # For now just be a pass-through controller.
        set_servo_pulse(0, int((state.get('rx', 0.5) - 0.5) * 2000 + 1500))
        set_servo_pulse(1, int((0.5 - state.get('ry', 0.5)) * 2000 + 1500))
        set_servo_pulse(2, int((state.get('rz', 0.5) - 0.5) * 2000 + 1500))
        time.sleep(0.01)


if __name__ == '__main__':
    jt = JoystickThread()
    jt.start()
    pwm = Adafruit_PCA9685.PCA9685()
    pwm.set_pwm_freq(60)
    motion_loop(jt, pwm)
