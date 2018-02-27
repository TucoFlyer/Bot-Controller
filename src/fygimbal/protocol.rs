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
    pub const MOTOR_POWER : u8 = 0x03;
    pub const CAPTURE_GYRO_DRIFT_COMPENSATION : u8 = 0x04;
    pub const SAVE_VALUES : u8 = 0x05;
    pub const GET_VALUE : u8 = 0x06;
    pub const SET_ACCEL_CORRECTION : u8 = 0x07;
    pub const SET_VALUE : u8 = 0x08;
    pub const GET_ACCEL_CORRECTIONS : u8 = 0x09;
    pub const ATTACH_MCU : u8 = 0x0a;
    pub const ATTACH_HOST : u8 = 0x0b;
    pub const CAPTURE_CALIBRATION_ANGLE : u8 = 0x0c;
    pub const INTERNAL_MCU1_TO_MCU0 : u8 = 0x0d;
    pub const INTERNAL_JOYSTICK_TOGGLE : u8 = 0x30;
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
    pub const Z : u8 = 0x00;
    pub const X : u8 = 0x01;
    pub const Y : u8 = 0x02;
}

#[allow(dead_code)]
pub mod motor_mode {
    pub const OFF : u8 = 0;
    pub const NORMAL : u8 = 1;
    pub const CURRENT_CONTROL_SIMULATED_ANGLE_OSCILLATOR : u8 = 2;
    pub const VELOCITY_CONTROL : u8 = 3;
    pub const CURRENT_CONTROL : u8 = 4;
    pub const ONESHOT_VELOCITY_CONTROL : u8 = 5;
    pub const CURRENT_CONTROL_FIXED_ANGLE : u8 = 6;
}

#[allow(dead_code)]
pub mod motor_error {
    pub const ERROR_CURRENT_THRESHOLD : i16 = 1<<3;
    pub const CALIBRATED_ACCEL_NEAR_ZERO : i16 = 1<<15;
}

#[allow(dead_code)]
pub mod motor_status {
    pub const LATCH_INPUT_UNK : i16 = 1<<0;
    pub const LATCH_STATE_UNK : i16 = 1<<1;
    pub const POWER_ON : i16 = 1<<2;
    pub const USER_CURRENT_THRESHOLD : i16 = 1<<9;
}

#[allow(dead_code)]
pub mod imu_type {
    pub const IMU_TYPE_68 : u8 = 0x68;
    pub const MPU6050 : u8 = 0x70;
}

#[allow(dead_code)]
pub mod values {
    pub const CHECKSUM : u8 = 0x00;
    pub const MOTOR_ERROR_FLAGS : u8 = 0x01;
    pub const MOTOR_STATUS_FLAGS : u8 = 0x02;
    pub const CONTROLLER_VELOCITY_INPUT : u8 = 0x03;
    pub const GYRO_ANGULAR_RATE : u8 = 0x04;
    pub const MOTOR_FILTERED_CURRENT_TARGET : u8 = 0x05;
    pub const MOTOR_FILTERED_CURRENT : u8 = 0x06;
    pub const SUPPLY_VOLTAGE : u8 = 0x07;
    pub const FOLLOW_INTEGRATOR : u8 = 0x08;
    pub const STABILIZER_VELOCITY_INPUT : u8 = 0x09;
    pub const MOTOR_MODE : u8 = 0x18;
    pub const MOTOR_ANGLE_OFFSET : u8 = 0x27;
    pub const MOTOR_ANGLE : u8 = 0x28;
    pub const ENCODER_ANGLE : u8 = 0x2c;
    pub const MOTOR_SHUTDOWN_ERROR_FLAGS : u8 = 0x2e;
    pub const MOTOR_CURRENT_TARGET : u8 = 0x47;
    pub const MOTOR_CURRENT : u8 = 0x48;
    pub const FOLLOW_INTEGRATOR_OFFSET : u8 = 0x4b;
    pub const FOLLOW_ANGLE_ERROR : u8 = 0x4c;
    pub const CALIBRATION_ANGLE_0_CENTER : u8 = 0x4d;
    pub const MANUAL_MOTOR_CURRENT_INPUT : u8 = 0x4e;
    pub const SIMULATED_ANGLE_OSCILLATOR_RATE : u8 = 0x4f;
    pub const MANUAL_MOTOR_VELOCITY_INPUT : u8 = 0x50;
    pub const CALIBRATION_GYRO_DRIFT_WAS_SET_AT_STARTUP_FLAG : u8 = 0x58;
    pub const CALIBRATION_GYRO_DRIFT_Z : u8 = 0x59;
    pub const CALIBRATION_GYRO_DRIFT_X : u8 = 0x5a;
    pub const CALIBRATION_GYRO_DRIFT_Y : u8 = 0x5b;
    pub const CALIBRATION_ACCEL_OFFSET_Z : u8 = 0x5c;
    pub const CALIBRATION_ACCEL_OFFSET_X : u8 = 0x5d;
    pub const CALIBRATION_ACCEL_OFFSET_Y : u8 = 0x5e;
    pub const FOLLOW_ENABLE_FLAG : u8 = 0x63;
    pub const CALIBRATION_ANGLE_1_MOTOR : u8 = 0x64;
    pub const FOLLOW_RATE : u8 = 0x65;
    pub const JOYSTICK_MODE : u8 = 0x66;
    pub const IMU_TYPE : u8 = 0x69;
    pub const MOTOR_CURRENT_ADC1 : u8 = 0x6f;
    pub const MOTOR_CURRENT_ADC2 : u8 = 0x70;
    pub const FIRMWARE_VERSION : u8 = 0x7f;
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

    pub fn set_accel_correction(axis: u8, value: i16) -> GimbalPacket {
        // IMU correction is also part of the normal "value" config space,
        // but writing to these slots must also update the internal
        // floating-point version of these values.

        let mut data = Vec::new();
        data.write_u8(axis).unwrap();
        data.write_i16::<LittleEndian>(value).unwrap();
        GimbalPacket {
            framing: GimbalFraming::Normal,
            command: super::cmd::SET_ACCEL_CORRECTION,
            target: super::target::IMU_ADJACENT,
            data,
        }
    }

    pub fn set_value(target: u8, index: u8, value: i16) -> GimbalPacket {
        match (target, index) {

            (super::target::IMU_ADJACENT, super::values::CALIBRATION_ACCEL_OFFSET_Z) =>
                set_accel_correction(super::imu_axis::Z, value),

            (super::target::IMU_ADJACENT, super::values::CALIBRATION_ACCEL_OFFSET_X) =>
                set_accel_correction(super::imu_axis::X, value),

            (super::target::IMU_ADJACENT, super::values::CALIBRATION_ACCEL_OFFSET_Y) =>
                set_accel_correction(super::imu_axis::Y, value),

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
