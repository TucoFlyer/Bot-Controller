'''
Low level Feiyu Gimbal protocol encoding and decoding
(No serial port dependency)
'''

import struct
import binascii

LONG_FORM = 0xaa55
SHORT_FORM = 0x5aa5

# LONG_FORM commands
#   cmd00  Init / Firmware version

# SHORT_FORM commands
#   cmd01  Control data?
#   cmd02
#   cmd03  On/Off
#   cmd04
#   cmd05  Save (per-axis?) config
#   cmd06  Get 16-bit value from 8-bit (per-axis?) config space
#   cmd07  Set 16-bit value in 8-bit (main?) config space
#   cmd08  Set 16-bit value in 8-bit (per-axis?) config space
#   cmd09  Unknown get, 12 bytes
#   cmd0a
#   cmd0b  Init PC control handshake
#   cmd0c  Capture center angles
#   cmd0d  IMU data?

class Packet:
    formats = {
        LONG_FORM: { 'len_struct': 'H', 'initial_crc_value': 0xffff },
        SHORT_FORM: { 'len_struct': 'B', 'initial_crc_value': 0x0000 },
    }

    def __init__(self, command, framing=SHORT_FORM, target=0, data=b''):
        if framing not in self.formats:
            raise ValueError('Unknown framing type 0x%04x', framing)
        self.command = command
        self.framing = framing
        self.target = target
        self.data = data

    def __repr__(self):
        return '<Pkt-%04X t=%02x cmd=%02x [%s]>' % (
            self.framing, self.target, self.command, binascii.b2a_hex(self.data).decode())

    @classmethod
    def crc(cls, framing, data):
        '''CRC-16 with an initial value that depends on the packet format'''
        return binascii.crc_hqx(data, cls.formats[framing]['initial_crc_value'])

    def format_option(self, name, default=None):
        return self.formats[self.framing].get(name, default)

    def pack(self):
        parts = [
            struct.pack('BB', self.target, self.command),
            struct.pack('<' + self.format_option('len_struct'), len(self.data)),
            self.data
        ]
        parts.append(struct.pack('<H', self.crc(self.framing, b''.join(parts))))
        return struct.pack('<H', self.framing) + b''.join(parts)


class PacketReceiver:
    packetClass = Packet

    def __init__(self):
        self.buffer = b''

    def parse(self, data):
        '''Yields a list of Packet instances, from 'data' or prior bytes.'''
        self.buffer += data
        while len(self.buffer) >= 2:
            framing = struct.unpack('<H', self.buffer[:2])[0]
            if framing not in self.packetClass.formats:
                self.buffer = self.buffer[1:]
                continue

            header_format = '<BB' + self.packetClass.formats[framing]['len_struct']
            header_len = struct.calcsize(header_format)
            header = self.buffer[2:2+header_len]
            if len(self.buffer) < header_len + 4:
                break
            target, command, data_len = struct.unpack(header_format, header)

            if len(self.buffer) < header_len + data_len + 4:
                break
            data = self.buffer[2+header_len:2+header_len+data_len]
            rx_crc = struct.unpack('<H', self.buffer[2+header_len+data_len:4+header_len+data_len])[0]
            self.buffer = self.buffer[4+header_len+data_len:]

            calc_crc = self.packetClass.crc(framing, header + data)
            if rx_crc != calc_crc:
                print("CRC mismatch, received %04x and expected %04x" % (rx_crc, calc_crc))
                continue

            yield Packet(command, framing, target, data)

