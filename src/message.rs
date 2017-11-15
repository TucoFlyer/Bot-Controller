use vecmath::*;
use serde_json::Value;
use std::time::Instant;
use config::{Config, ControllerMode};
use fygimbal::GimbalPacket;

pub const TICK_HZ : u32 = 250;

/// Commands can be sent unmodified by an authenticated websockets client
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Command {
    SetMode(ControllerMode),
    ManualControlReset,
    ManualControlValue(ManualControlAxis, f32),
    CameraObjectDetection(CameraDetectedObjects),
    CameraRegionTracking(CameraTrackedRegion),
    GimbalMotorEnable(bool),
    GimbalPacket(GimbalPacket),
    GimbalValueWrite(GimbalValueData),
    GimbalValueRequests(Vec<GimbalValueRequest>),
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
    GimbalControlStatus(GimbalControlStatus),
    GimbalValue(GimbalValueData, GimbalValueOp),
    UnhandledGimbalPacket(GimbalPacket),
    CameraOverlayScene(Vec<OverlayRect>),
    CameraInitTrackedRegion(Vector4<f32>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GimbalValueAddress {
    pub target: u8,
    pub index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GimbalValueData {
    pub addr: GimbalValueAddress,
    pub value: i16,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GimbalValueRequest {
    pub addr: GimbalValueAddress,
    pub scope: GimbalRequestScope,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GimbalRequestScope {
    Once,
    Continuous,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum GimbalValueOp {
    ReadComplete,
    WriteComplete,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GimbalControlStatus {
    pub angles: Vector2<i16>,
    pub rates: Vector2<i16>,
    pub tracking_p_rates: Vector2<f32>,
    pub tracking_i_rates: Vector2<f32>,
    pub yaw_gain_activations: Vec<f32>,
    pub pitch_gain_activations: Vec<f32>,
    pub drift_compensation: Vector2<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct OverlayRect {
    /// Texels with [0,0] at top left
    pub src: Vector4<i32>,
    /// Arbitrary coordinates centered on zero with horizontal from [-1,1] and aspect correct
    pub dest: Vector4<f32>,
    pub rgba: Vector4<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CameraTrackedRegion {
    /// Horizontal camera extents [-1,1], aspect correct, Y+ down
    pub rect: Vector4<f32>,
    /// Previous frame's corresponding rectangle.
    pub previous_rect: Vector4<f32>,
    /// Peak to side ratio (tracking quality)
    pub psr: f32,
    /// Number of frames since last tracking reset
    pub age: u32,
    /// Frame serial number associated with this tracking result
    pub frame: u32,
    /// Time spent running the tracker to get this result
    pub tracker_nsec: u64,
}

impl CameraTrackedRegion {
    pub fn new() -> CameraTrackedRegion {
        CameraTrackedRegion {
            rect: [0.0, 0.0, 0.0, 0.0],
            previous_rect: [0.0, 0.0, 0.0, 0.0],
            psr: 0.0,
            age: 0,
            frame: 0,
            tracker_nsec: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rect[2] <= 0.0 || self.rect[3] <= 0.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CameraDetectedObjects {
    pub frame: u32,
    pub detector_nsec: u64,
    pub objects: Vec<CameraDetectedObject>,
}

impl CameraDetectedObjects { 
    pub fn new() -> CameraDetectedObjects {
        CameraDetectedObjects {
            frame: 0,
            detector_nsec: 0,
            objects: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CameraDetectedObject {
    pub rect: Vector4<f32>,
    pub prob: f32,
    pub label: String,
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
    pub accelerometer: Vector3<i16>,
    pub magnetometer: Vector3<i16>,
    pub gyroscope: Vector3<i16>,
    pub euler_angles: Vector3<i16>,
    pub quaternion: Vector4<i16>,
    pub linear_accel: Vector3<i16>,
    pub gravity: Vector3<i16>,
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
    pub p_filter_param: f32,    // IIR filter parameter in range [0,1] for position error, 0=slow 1=fast
    pub i_decay_param: f32,     // Exponential decay for the integral parameter, 0=slow 1=fast
    pub d_filter_param: f32,    // IIR filter parameter in range [0,1] for velocity error, 0=slow 1=fast
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WinchDeadband {
    pub position: i32,          // How close is close enough when stopped?
    pub velocity: f32,          // By "stopped", we mean under this instantaneous velocity
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WinchCommand {
    pub position: i32,
    pub force: ForceCommand,
    pub pid: PIDGains,
    pub deadband: WinchDeadband,
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
    pub pos_err_filtered: f32,      // Low-pass-filtered position error
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
