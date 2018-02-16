pub const NUM_AXES : usize = 3;
pub const NUM_VALUES : usize = 128;

#[allow(dead_code)]
pub mod bootloader_cmd {
    pub const VERSION : u8 = 0x00;
    pub const INTERRUPT_BOOT : u8 = 0x01;
    pub const WRITE_BLOCK : u8 = 0x02;
    pub const WRITE_BLOCK_ACK : u8 = 0x03;
    pub const NEXT_MCU : u8 = 0x07;
    pub const NEXT_MCU_ACK : u8 = 0x08;
}

#[allow(dead_code)]
pub mod cmd {
    pub const INTERNAL_MCU2_TO_MCU1 : u8 = 0x00;
    pub const INTERNAL_JOYSTICK_DATA : u8 = 0x01;
    pub const UNK_COMMAND_2 : u8 = 0x02;
    pub const MOTOR_POWER : u8 = 0x03;
    pub const UNK_IMU_CALIBRATE_4 : u8 = 0x04;
    pub const SAVE_VALUES : u8 = 0x05;
    pub const GET_VALUE : u8 = 0x06;
    pub const SET_ATTITUDE_CORRECTION : u8 = 0x07;
    pub const SET_VALUE : u8 = 0x08;
    pub const GET_ATTITUDE_CORRECTION : u8 = 0x09;
    pub const UNK_COMMAND_A : u8 = 0x0a;
    pub const SERIAL_ATTACH : u8 = 0x0b;
    pub const CALIBRATE : u8 = 0x0c;
    pub const INTERNAL_MCU1_TO_MCU0 : u8 = 0x0d;
}

#[allow(dead_code)]
pub mod target {
    pub const YAW : u8 = 0x00;
    pub const ROLL : u8 = 0x01;
    pub const PITCH : u8 = 0x02;
    pub const IMU_ADJACENT : u8 = 0x02;
    pub const HOST : u8 = 0x03;
}

#[allow(dead_code)]
pub mod imu_axis {
    pub const ROLL : u8 = 0x00;
    pub const PITCH : u8 = 0x01;
    pub const YAW : u8 = 0x02;
}

#[allow(dead_code)]
pub mod values {
    pub const CONTROL_RATE : u8 = 0x03;
    pub const ENCODER_ANGLES : u8 = 0x2c;
    pub const CENTER_CALIBRATION : u8 = 0x4D;
    pub const ATTITUDE_CORRECTION_ROLL : u8 = 0x5C;
    pub const ATTITUDE_CORRECTION_PITCH : u8 = 0x5D;
    pub const ATTITUDE_CORRECTION_YAW : u8 = 0x5E;
    pub const MOTOR_CALIBRATION : u8 = 0x64;
}

pub mod pack {
    use fygimbal::framing::{GimbalFraming, GimbalPacket};
    use byteorder::{WriteBytesExt, LittleEndian};

    pub fn get_value(target: u8, index: u8) -> GimbalPacket {
        GimbalPacket {
            framing: GimbalFraming::Normal,
            command: super::cmd::GET_VALUE,
            target,
            data: vec![index],
        }
    }

    pub fn set_attitude_correction(axis: u8, value: i16) -> GimbalPacket {
        // Attitude correction is also part of the normal "value" config space,
        // but writing to these attitude correction slots must also update the
        // internal floating-point version of these values. MCU2 manages the
        // attitude correction for all axes.

        let mut data = Vec::new();
        data.write_u8(axis).unwrap();
        data.write_i16::<LittleEndian>(value).unwrap();
        GimbalPacket {
            framing: GimbalFraming::Normal,
            command: super::cmd::SET_ATTITUDE_CORRECTION,
            target: super::target::IMU_ADJACENT,
            data,
        }
    }

    pub fn set_value(target: u8, index: u8, value: i16) -> GimbalPacket {
        match (target, index) {

            (super::target::IMU_ADJACENT, super::values::ATTITUDE_CORRECTION_ROLL) =>
                set_attitude_correction(super::imu_axis::ROLL, value),

            (super::target::IMU_ADJACENT, super::values::ATTITUDE_CORRECTION_PITCH) =>
                set_attitude_correction(super::imu_axis::PITCH, value),

            (super::target::IMU_ADJACENT, super::values::ATTITUDE_CORRECTION_YAW) =>
                set_attitude_correction(super::imu_axis::YAW, value),

            (target, index) => {
                let mut data = Vec::new();
                data.write_u16::<LittleEndian>(index as u16).unwrap();
                data.write_i16::<LittleEndian>(value).unwrap();
                GimbalPacket {
                    framing: GimbalFraming::Normal,
                    command: super::cmd::SET_VALUE,
                    target, data,
                }
            },
        }
    }

    pub fn motor_power(target: u8, value: u8) -> GimbalPacket {
        GimbalPacket {
            framing: GimbalFraming::Normal,
            command: super::cmd::MOTOR_POWER,
            target,
            data: vec![value],
        }
    }
}

pub mod unpack {
    use std::io::{Result, Cursor};
    use fygimbal::framing::{GimbalFraming, GimbalPacket};
    use byteorder::{ReadBytesExt, LittleEndian};

    pub fn get_value(packet: &GimbalPacket) -> Result<i16> {
        assert!(packet.framing == GimbalFraming::Normal);
        assert!(packet.command == super::cmd::GET_VALUE);
        let mut reader = Cursor::new(&packet.data);
        reader.read_i16::<LittleEndian>()
    }
}
