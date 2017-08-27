//! Command and status busses shared between components and threads

use multiqueue;
use serde_json::Value;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use config::{Config, ControllerMode};
use vecmath::{I16Vec3, I16Vec4};

#[derive(Clone)]
pub struct Bus {
    pub sender: multiqueue::BroadcastSender<TimestampedMessage>,
    pub receiver: multiqueue::BroadcastReceiver<TimestampedMessage>,
    pub config: Arc<Mutex<Config>>,
}

impl Bus {
    pub fn new(config: Config) -> Bus {
        let (sender, receiver) = multiqueue::broadcast_queue(512);
        let config = Arc::new(Mutex::new(config));
        Bus { sender, receiver, config }
    }
}

/// Commands can be sent unmodified by an authenticated websockets client
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Command {
    SetMode(ControllerMode),
    ManualControlReset,
    ManualControlValue(ManualControlAxis, f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimestampedMessage {
    pub timestamp: Instant,
    pub message: Message,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Message {
    Command(Command),
    FlyerSensors(FlyerSensors),
    WinchStatus(usize, WinchStatus),
    UpdateConfig(Value),
    ConfigIsCurrent(Config),
}

impl Message {
    pub fn timestamp(self) -> TimestampedMessage {
        TimestampedMessage {
            timestamp: Instant::now(),
            message: self
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManualControlAxis {
    CameraYaw,
    CameraPitch,
    RelativeX,
    RelativeY,
    RelativeZ,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct XBandTelemetry {
    pub edge_count: u32,
    pub speed_measure: u32,
    pub measure_count: u32,
}

const NUM_LIDAR_SENSORS : usize = 4;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LIDARTelemetry {
    pub ranges: [u32; NUM_LIDAR_SENSORS],
    pub counters: [u32; NUM_LIDAR_SENSORS],
}

const NUM_ANALOG_SENSORS : usize = 8;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AnalogTelemetry {
    pub values: [u32; NUM_ANALOG_SENSORS],
    pub counter: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IMUTelemetry {
    pub accelerometer: I16Vec3,
    pub magnetometer: I16Vec3,
    pub gyroscope: I16Vec3,
    pub euler_angles: I16Vec3,
    pub quaternion: I16Vec4,
    pub linear_accel: I16Vec3,
    pub gravity: I16Vec3,
    pub temperature: i8,
    pub calib_stat: i8,
    pub counter: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FlyerSensors {
    pub xband: XBandTelemetry,
    pub lidar: LIDARTelemetry,
    pub analog: AnalogTelemetry,
    pub imu: IMUTelemetry,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ForceTelemetry {
    pub measure: i32,           // Uncalibrated, (+) = increasing tension
    pub filtered: f32,          // Same units, just low-pass filtered prior to limit testing
    pub counter: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ForceCommand {
    pub filter_param: f32,      // IIR filter parameter in range [0,1] for force sensor, 0=slow 1=fast
    pub neg_motion_min: f32,    // Uncalibrated load cell units, no negative motion below
    pub pos_motion_max: f32,    // Uncalibrated load cell units, no positive motion above this filtered force value
    pub lockout_below: f32,     // Uncalibrated load cell units, no motion at all below
    pub lockout_above: f32,     // Uncalibrated load cell units, no motion at all above
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PIDGains {
    pub gain_p: f32,            // PWM strength proportional to position error
    pub gain_i: f32,            // PWM strength proportional to integral of position error
    pub gain_d: f32,            // PWM gain proportional to velocity error
    pub d_filter_param: f32,    // IIR filter parameter in range [0,1] for velocity error, 0=slow 1=fast
    pub i_decay_param: f32,     // Exponential decay for the integral parameter, 0=slow 1=fast
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WinchCommand {
    pub force: ForceCommand,
    pub pid: PIDGains,
    pub position: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WinchSensors {
    pub force: ForceTelemetry,
    pub position: i32,              // Integrated position in encoder units, from hardware
    pub velocity: f32,              // Calculated instantaneous velocity at each tick
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WinchPWM {
    pub total: f32,                 // PWM calculated by the PID loop, clamped to [-1, 1]
    pub p: f32,                     // Just the contribution from proportional gain
    pub i: f32,                     // Just the contribution from integral gain
    pub d: f32,                     // Just the contribution from derivative gain
    pub quant: i16,                 // PWM state after quantizing into clock ticks
    pub enabled: i16,               // Is the H-bridge enabled? Can be turned off by halt conditions.
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WinchMotorControl {
    pub pwm: WinchPWM,
    pub position_err: i32,          // Instantaneous position error
    pub pos_err_integral: f32,      // Accumulated integral of the position error, reset by halt watchdog
    pub vel_err_inst: f32,          // Instantaneous velocity error
    pub vel_err_filtered: f32,      // Low-pass-filtered velocity error
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WinchStatus {
    pub command_counter: u32,
    pub tick_counter: u32,
    pub command: WinchCommand,
    pub sensors: WinchSensors,
    pub motor: WinchMotorControl
}
