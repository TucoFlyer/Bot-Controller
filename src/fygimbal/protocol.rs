mod bootloader_cmd {
    pub const VERSION : u8 = 0x00;
    pub const INTERRUPT_BOOT : u8 = 0x01;
    pub const WRITE_BLOCK : u8 = 0x02;
    pub const WRITE_BLOCK_ACK : u8 = 0x03;
    pub const NEXT_MCU : u8 = 0x07;
    pub const NEXT_MCU_ACK : u8 = 0x08;
}

mod cmd {
    pub const INTERNAL_JOYSTICK_DATA : u8 = 0x01;
    pub const MOTOR_POWER : u8 = 0x03;
    pub const SAVE_VALUES : u8 : 0x05;
    pub const GET_VALUE : u8 = 0x06;
    pub const SET_VALUE : u8 = 0x08;
    pub const SERIAL_ATTACH : u8 = 0x0b;
    pub const CALIBRATE : u8 = 0x0c;
    pub const INTERNAL_IMU_DATA : u8 = 0x0d;
}

mod target {
    pub const YAW : u8 = 0x00;
    pub const ROLL : u8 = 0x01;
    pub const PITCH : u8 = 0x02;
    pub const HOST : u8 = 0x03;
}

mod values {
    pub const CONTROL_RATE : u8 = 0x03;
    pub const ENCODER_ANGLES : u8 = 0x2c;
    pub const CENTER_CALIBRATION : u8 = 0x4D;
    pub const MOTOR_CALIBRATION : u8 = 0x64;
}