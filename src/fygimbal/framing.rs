use std::io;
use std::io::{Cursor, Write};
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt, ReadBytesExt};
use crc16;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GimbalPacket {
    pub framing: GimbalFraming,
    pub command: u8,
    pub target: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum GimbalFraming {
    Bootloader,
    Normal,
}

impl GimbalPacket {
    pub fn write_to(&self, wr: &mut io::Write) -> io::Result<()> {
        wr.write(&self.framing.to_bytes())?;
        self.write_header(wr)?;
        wr.write(&self.data)?;
        wr.write(&self.crc_bytes())?;
        Ok(())
    }

    fn write_header(&self, wr: &mut io::Write) -> io::Result<()> {
        wr.write_u8(self.target)?;
        wr.write_u8(self.command)?;
        self.framing.write_length(wr, self.data.len())?;
        Ok(())
    }

    fn crc_bytes(&self) -> [u8; 2] {
        let mut body = Vec::new();
        self.write_header(&mut body).unwrap();
        body.write(&self.data).unwrap();
        self.framing.crc_bytes(&body)
    }

    fn parse(buffer: &[u8]) -> (&[u8], Option<GimbalPacket>) {
        let (remainder, framing) = GimbalFraming::parse(buffer);
        match framing {
            None => (remainder, None),
            Some(framing) => {
                if remainder.len() <= 2 {
                    // Wait for full header
                    (buffer, None)
                } else {
                    let (header, remainder) = remainder.split_at(2);
                    let target = header[0];
                    let command = header[1];
                    let (remainder, length) = framing.parse_length(remainder);
                    match length {
                        None => (buffer, None),
                        Some(length) => {
                            let length_with_crc = length + 2;

                            if remainder.len() < length_with_crc {
                                // Wait for more data
                                (buffer, None)
                            } else {
                                let (data, remainder) = remainder.split_at(length_with_crc);
                                let (data, stored_crc) = data.split_at(length);
                                let data = data.to_vec();
                                let packet = GimbalPacket { framing, command, target, data };
                                if packet.crc_bytes() == stored_crc {
                                    (remainder, Some(packet))
                                } else {
                                    // Ignore whole packet, bad CRC
                                    (remainder, None)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl GimbalFraming {
    fn to_bytes(&self) -> [u8; 2] {
        match self {
            &GimbalFraming::Bootloader => [0x55, 0xaa],
            &GimbalFraming::Normal => [0xa5, 0x5a],
        }
    }

    fn from_bytes(bytes: &[u8]) -> Option<GimbalFraming> {
        if bytes.len() >= 2 {
            if bytes[0..2] == GimbalFraming::Bootloader.to_bytes() {
                Some(GimbalFraming::Bootloader)
            } else if bytes[0..2] == GimbalFraming::Normal.to_bytes() {
                Some(GimbalFraming::Normal)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse(buffer: &[u8]) -> (&[u8], Option<GimbalFraming>) {
        if buffer.len() >= 2 {
            // Look for framing sequence at each byte offset
            for ignored in 0..(buffer.len() - 2) {
                if let Some(result) = GimbalFraming::from_bytes(&buffer[ignored..]) {
                    return (&buffer[ignored + 2 ..], Some(result));
                }
            }
        }

        if buffer.len() >= 1 {
            // Discard junk but save the last byte, in case it's part of an incomplete framing
            return (&buffer[buffer.len()-1..], None);
        }

        (buffer, None)
    }

    fn write_length(&self, wr: &mut io::Write, len: usize) -> io::Result<()> {
        match self {
            &GimbalFraming::Bootloader => {
                assert!(len <= 0xFFFF);
                wr.write_u16::<LittleEndian>(len as u16)?;
            },
            &GimbalFraming::Normal => {
                assert!(len <= 0xFF);
                wr.write_u8(len as u8)?;
            },
        }
        Ok(())
    }

    fn parse_length<'a>(&self, buffer: &'a [u8]) -> (&'a [u8], Option<usize>) {
        let (offset, result) = {
            let mut cursor = Cursor::new(buffer);
            let result = match self {
                &GimbalFraming::Bootloader => match cursor.read_u16::<LittleEndian>() {
                    Err(_) => None,
                    Ok(len) => Some(len as usize),
                },
                &GimbalFraming::Normal => match cursor.read_u8() {
                    Err(_) => None,
                    Ok(len) => Some(len as usize),
                },
            };
            (cursor.position() as usize, result)
        };
        (&buffer[offset..], result)
    }

    fn calculate_crc(&self, data: &[u8]) -> u16 {
        match self {
            &GimbalFraming::Bootloader => crc16::State::<crc16::CCITT_FALSE>::calculate(data),
            &GimbalFraming::Normal => crc16::State::<crc16::XMODEM>::calculate(data),
        }
    }

    fn crc_bytes(&self, data: &[u8]) -> [u8; 2] {
        let mut bytes = [0; 2];
        LittleEndian::write_u16(&mut bytes, self.calculate_crc(data));
        bytes
    }
}

#[derive(Debug)]
pub struct PacketReceiver {
    buffer: Vec<u8>,
}

impl PacketReceiver {
    pub fn new() -> PacketReceiver {
        PacketReceiver { 
            buffer: Vec::new(),
        }
    }
}

impl io::Write for PacketReceiver {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Iterator for PacketReceiver {
    type Item = GimbalPacket;

    fn next(&mut self) -> Option<GimbalPacket> {
        let (buffer, packet) = {
            let (remainder, packet) = GimbalPacket::parse(&self.buffer);
            (remainder.to_vec(), packet)
        };
        self.buffer = buffer;
        packet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_boot_cmd01() {
        let packet = GimbalPacket {
            framing: GimbalFraming::Bootloader,
            command: 1,
            target: 0,
            data: vec![]
        };
        let mut v = Vec::new();
        packet.write_to(&mut v).unwrap();
        assert_eq!(v, vec![0x55, 0xaa, 0x00, 0x01, 0x00, 0x00, 0xf0, 0xb3]);
    }

    #[test]
    fn decode_boot_cmd01() {
        let mut recv = PacketReceiver::new();
        recv.write(&[0x12, 0x34, 0x55, 0xaa, 0x00, 0x01, 0x00, 0x00, 0xf0, 0xb3, 0x55, 0xaa]).unwrap();
        assert_eq!(recv.next(), Some(GimbalPacket {
            framing: GimbalFraming::Bootloader,
            command: 1,
            target: 0,
            data: vec![]
        }));
        assert_eq!(recv.buffer, vec![0x55, 0xaa]);
    }

    #[test]
    fn decode_boot_cmd01_bad_crc() {
        let mut recv = PacketReceiver::new();
        recv.write(&[0x12, 0x34, 0x55, 0xaa, 0x00, 0x01, 0x00, 0x00, 0xff, 0xb3, 0x99, 0xaa]).unwrap();
        assert_eq!(recv.next(), None);
        assert_eq!(recv.buffer, vec![0x99, 0xaa]);
    }

    #[test]
    fn encode_version_packet() {
        let packet = GimbalPacket {
            framing: GimbalFraming::Normal,
            command: 8,
            target: 0,
            data: vec![0x65, 0x00, 0x2c, 0x01]
        };
        let mut v = Vec::new();
        packet.write_to(&mut v).unwrap();
        assert_eq!(v, vec![0xa5, 0x5a, 0x00, 0x08, 0x04, 0x65, 0x00, 0x2c, 0x01, 0x79, 0x32]);
    }

    #[test]
    fn decode_version_packet() {
        let mut recv = PacketReceiver::new();
        recv.write(&[0xa5, 0x5a, 0x00, 0x08, 0x04, 0x65, 0x00, 0x2c, 0x01, 0x79, 0x32, 0x00, 0x00, 0x00]).unwrap();
        assert_eq!(recv.next(), Some(GimbalPacket {
            framing: GimbalFraming::Normal,
            command: 8,
            target: 0,
            data: vec![0x65, 0x00, 0x2c, 0x01]
        }));
        assert_eq!(recv.buffer, vec![0x00, 0x00, 0x00]);
    }

}
