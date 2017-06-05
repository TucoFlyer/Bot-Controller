#!/usr/bin/env python3
from flyergimbal import GimbalPort

gimbal = GimbalPort(verbose=False)

slots = [None] * 128
nRange = range(128)

# Clear
gimbal.waitConnect()
print("\x1b[2J")

def update(n, sgr=[0]):
    cols = 4
    rows = 128/cols
    print("\x1b[%sm\x1b[%d;%dH[%02x] %-30r\x1b[0m" % (
        ';'.join(map(str, sgr)),
        3 + (n%rows), 3 + (n//rows) * 35,
        n, slots[n]))

# Initial values
for n in nRange:
    slots[n] = gimbal.getVectorParam(n)
    update(n)

while True:
    for n in nRange:
        previous = slots[n]
        current = gimbal.getVectorParam(n)
        if previous != current:
            slots[n] = current
            update(n, sgr=(1,32))
