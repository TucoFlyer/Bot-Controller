#!/usr/bin/env python3
#
# Just a little joystick interface for linux evdev.
# Slurps incoming events into Python objects on a background thread.
# 

import evdev
import threading
import time


def deadzone(v, width=0.3):
    if v > width/2:
        return (v - width/2) / (1.0 - width)
    if v < -width/2:
        return (v + width/2) / (1.0 - width)
    return 0


class JoystickThread(threading.Thread):
    def __init__(self, device=None):
        threading.Thread.__init__(self)
        self.device = device or self._default_joystick()
        self.axes = {}
        self._pending = {}
        for axis, info in self.device.capabilities().get(evdev.ecodes.EV_ABS, []):
            self.axes[axis] = (info, [None])
        self.setDaemon(True)
        self.start()

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
                s[evdev.ecodes.ABS[axis].lower().split('_')[1]] = (mapped - 0.5) * 2.0
        return s


def main():
    js = JoystickThread()
    while True:
        print(js.state(), js.axes)
        time.sleep(0.1)

if __name__ == '__main__':
    main()
