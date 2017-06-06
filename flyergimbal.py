'''
UDP interface to the Feiyu Tech gimbal on Tuco Flyer
'''

import struct
import threading
import traceback
import queue
import time

import fyproto
import socket

MSG_GIMBAL = b'\x01'
MSG_FLYER_SENSORS = b'\x02'
PORT = 9024
CONTROLLER = ('10.32.0.1', PORT)
FLYER = ('10.32.0.8', PORT)


class Timeout(Exception):
    '''Timed out while waiting for a response from the gimbal'''
    pass


class ReceiverThread(threading.Thread):
    receiverClass = fyproto.PacketReceiver

    def __init__(self, socket, callback, verbose=False):
        threading.Thread.__init__(self)
        self.socket = socket
        self.callback = callback
        self.running = True
        self.verbose = verbose
        self.receiver = self.receiverClass()
        self.setDaemon(True)

    def run(self):
        while self.running:
            packet, src = self.socket.recvfrom(4096)

            if src == FLYER and len(packet) > 0 and packet[:1] == MSG_GIMBAL:
                for packet in self.receiver.parse(packet[1:]):
                    if self.verbose:
                        print("RX %s" % packet)
                    try:
                        self.callback(packet)
                    except:
                        traceback.print_exc()

            elif src == FLYER and len(packet) > 0 and packet[:1] == MSG_FLYER_SENSORS:
                print("Flyer Sensors: ", struct.unpack('<' + 'i' * ((len(packet)//4)), packet[1:]))

            else:
                print("UDP from ", srcaddr, binascii.b2a_hex(pkt))


class GimbalPort:
    receiverThreadClass = ReceiverThread
    axes = range(3)
    transactionRetries = 15
    transactionTimeout = 2.0
    connectTimeout = 10.0

    def __init__(self, verbose=True, connected=None):
        self.verbose = verbose
        self.version = None

        self.connectedCV = threading.Condition()
        self.responseQueue = queue.Queue()
        self._transactionLock = threading.Lock()

        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.bind(CONTROLLER)

        self.rx = self.receiverThreadClass(self.socket, callback=self._receive, verbose=self.verbose)
        self.rx.start()

        if connected is None:
            self.connected = True
            self.connected = self._testForExistingConnection()
        else:
            self.connected = connected
        if self.verbose:
            if self.connected:
                print("Already connected to gimbal, version %s" % self.version)
            else:
                print("Waiting for gimbal to power on")

    def close(self):
        self.rx.running = False
        self.rx.join()
        self.port.close()

    def _testForExistingConnection(self):
        if self.verbose:
            print("Checking for existing connection")
        try:
            paramVersion = self.getParam(target=0, number=0x7f, retries=0, timeout=0.1)
            self.version = self.version or (paramVersion / 100)
            return True
        except Timeout:
            return False

    def flush(self, timeout=None):
        # Perform an unnecessary 'get' to ensure all other commands have been seen
        self.getParam(target=0, number=0x7f, retries=0, timeout=timeout)

    def send(self, packet):
        self.waitConnect()
        if self.verbose:
            print("TX %s" % packet)
        self.socket.sendto(MSG_GIMBAL + packet.pack(), FLYER)

    def waitConnect(self, timeout=None):
        if self.connected:
            return
        timeout = timeout or self.connectTimeout
        with self.connectedCV:
            self.connectedCV.wait_for(lambda: self.connected, timeout=timeout)
        if not self.connected:
            raise Timeout()

    def _receive(self, packet):
        '''One packet received by the ReceiverThread.
           This immediately handles some packets,
           and queues responses to be picked up later by _waitResponse().
           '''
        if packet.framing == fyproto.LONG_FORM:
            if packet.command == 0x00:
                self.cmd00 = packet
                _unknown, version = struct.unpack("<HH", packet.data)
                self.version = version / 100.0
                return

        if packet.framing == fyproto.SHORT_FORM:
            if packet.command == 0x0B:
                if self.verbose:
                    print("Connecting to gimbal, firmware version %s" % self.version)
                with self.connectedCV:
                    self.connected = True
                    self.send(fyproto.Packet(target=0, command=0x0b, data=bytes([0x01])))
                    self.connectedCV.notify_all()
                return

            if packet.target == 0x03:
                self.responseQueue.put(packet)
                return

    def _waitResponse(self, command, timeout):
        '''Wait for a response to the indicated command, with a timeout.'''
        deadline = timeout and (time.time() + timeout)
        try:
            while True:
                timeout = deadline and max(0, deadline - time.time())
                packet = self.responseQueue.get(timeout=timeout)
                if packet.command == command:
                    return packet
                if self.verbose:
                    print("Ignored response %r" % packet)
        except queue.Empty:
            raise Timeout()

    def transaction(self, packet, timeout=None, retries=None):
        '''Send a packet, and wait for the corresponding response, with retry on timeout'''
        self.waitConnect()
        if timeout is None:
            timeout = self.transactionTimeout
        if retries is None:
            retries = self.transactionRetries
        while True:
            try:
                with self._transactionLock:
                    self.send(packet)
                    return self._waitResponse(packet.command, timeout=timeout)
            except Timeout:
                retries -= 1
                if retries < 0:
                    raise

    def setMotors(self, enable, targets=axes):
        # Not sure if order matters
        for t in sorted(targets, reverse=True):
            self.send(fyproto.Packet(target=t, command=0x03, data=struct.pack('B', enable)))

        if enable:
            # Unknown
            self.setParam(target=2, number=0x67, value=1)

    def storeCalibrationAngle(self, num, targets=axes):
        for target in targets:
            p = fyproto.Packet(target=target, command=0x0c, data=struct.pack('B', num))
            self.transaction(p)

    def saveParams(self, targets=axes, timeout=None, retries=None):
        for target in targets:
            p = fyproto.Packet(target=target, command=0x05, data=b'\x00')
            r = self.transaction(p, timeout=timeout, retries=retries)
            if struct.unpack('<B', r.data)[0] != target:
                raise IOError("Failed to save parameters, response %r" % packet)
            if self.verbose:
                print("Saved params on MCU %d" % target)

    def getParam(self, target, number, fmt='h', timeout=None, retries=None):
        p = fyproto.Packet(target=target, command=0x06, data=struct.pack('B', number))
        r = self.transaction(p, timeout=timeout, retries=retries)
        return struct.unpack('<' + fmt, r.data)[0]

    def setParam(self, target, number, value, fmt='h'):
        self.send(fyproto.Packet(target=target, command=0x08, data=struct.pack('<BB' + fmt, number, 0, value)))

    def getVectorParam(self, number, targets=axes, timeout=None, retries=None):
        return tuple(self.getParam(t, number, timeout=timeout, retries=retries) for t in targets)

    def setVectorParam(self, number, value, targets=axes):
        for i, t in enumerate(targets):
            self.setParam(t, number, value[i])
