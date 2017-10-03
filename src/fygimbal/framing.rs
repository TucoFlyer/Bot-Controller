use std::io;
use std::io::Write;
use byteorder::{LittleEndian, WriteBytesExt};
use crc16;

pub struct GimbalPacket {
    framing: FramingType,
    command: u8,
    target: u8,
    data: Vec<u8>,
}

impl GimbalPacket {
    pub fn write_to(&self, wr: &mut io::Write) -> io::Result<()> {
        wr.write(&self.framing.to_bytes())?;
        let mut body = Vec::new();
        body.write_u8(self.target)?;
        body.write_u8(self.command)?;
        self.framing.write_length(&mut body, self.data.len())?;
        body.write(&self.data)?;
        wr.write(&body)?;
        wr.write_u16::<LittleEndian>(self.framing.calculate_crc(&body))?;
        Ok(())
    }
}

pub struct PacketReceiver {
    buffer: Vec<u8>,
}

impl PacketReceiver {
    fn new() -> PacketReceiver {
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
        loop {
            match FramingType::from_bytes(&self.buffer[..2]) {
                None => break,
                Some(framing) => {
                    return Some(GimbalPacket {
                        framing,
                        command: 0,
                        target: 0,
                        data: vec![],
                    });
                } 
            }
        }
        None
    }
}

pub enum FramingType {
    Bootloader,
    Normal,
}

impl FramingType {
    fn to_bytes(&self) -> [u8; 2] {
        match self {
            &FramingType::Bootloader => [0x55, 0xaa],
            &FramingType::Normal => [0xa5, 0x5a],
        }
    }

    fn from_bytes(bytes: &[u8]) -> Option<FramingType> {
        if bytes.len() >= 2 {
            if bytes[0..2] == FramingType::Bootloader.to_bytes() {
                Some(FramingType::Bootloader)
            } else if bytes[0..2] == FramingType::Normal.to_bytes() {
                Some(FramingType::Normal)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn write_length(&self, wr: &mut io::Write, len: usize) -> io::Result<()> {
        match self {
            &FramingType::Bootloader => {
                assert!(len <= 0xFFFF);
                wr.write_u16::<LittleEndian>(len as u16)?;
            },
            &FramingType::Normal => {
                assert!(len <= 0xFF);
                wr.write_u8(len as u8)?;
            },
        }
        Ok(())
    }

    fn calculate_crc(&self, data: &[u8]) -> u16 {
        match self {
            &FramingType::Bootloader => crc16::State::<crc16::CCITT_FALSE>::calculate(data),
            &FramingType::Normal => crc16::State::<crc16::XMODEM>::calculate(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_boot_cmd01() {
        let packet = GimbalPacket {
            framing: FramingType::Bootloader,
            command: 1,
            target: 0,
            data: vec![]
        };
        let mut v = Vec::new();
        packet.write_to(&mut v).unwrap();
        assert_eq!(v, vec![0x55, 0xaa, 0x00, 0x01, 0x00, 0x00, 0xf0, 0xb3]);
    }


    #[test]
    fn encode_version_packet() {
        let packet = GimbalPacket {
            framing: FramingType::Normal,
            command: 8,
            target: 0,
            data: vec![0x65, 0x00, 0x2c, 0x01]
        };
        let mut v = Vec::new();
        packet.write_to(&mut v).unwrap();
        assert_eq!(v, vec![0xa5, 0x5a, 0x00, 0x08, 0x04, 0x65, 0x00, 0x2c, 0x01, 0x79, 0x32]);
    }

}
