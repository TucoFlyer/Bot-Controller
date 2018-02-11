//! Bot configuration

use vecmath::*;
use atomicwrites;
use serde_json::{Value, from_value, to_value};
use serde_json;
use serde_yaml;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::NaiveTime;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Config {
    pub mode: ControllerMode,
    pub controller_addr: SocketAddr,
    pub flyer_addr: SocketAddr,
    pub web: WebConfig,
    pub metrics: Option<MetricsConfig>,
    pub params: BotParams,
    pub gimbal: GimbalConfig,
    pub overlay: OverlayConfig,
    pub vision: VisionConfig,
    pub winches: Vec<WinchConfig>,
    pub lighting: LightingConfig,
}

#[derive(Clone)]
pub struct SharedConfigFile {
    config: Arc<Mutex<Config>>,
    async_save_channel: Sender<Config>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ControllerMode {
    Halted,
    Normal,
    ManualFlyer,
    ManualWinch(usize),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WinchConfig {
    pub addr: SocketAddr,
    pub loc: Vector3<f32>,
    pub calibration: WinchCalibration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WinchCalibration {
    pub force_zero_count: f32,
    pub kg_force_per_count: f32,
    pub m_dist_per_count: f32,
}

impl WinchCalibration {
    pub fn force_to_kg(self: &WinchCalibration, counts: f32) -> f32 {
        self.kg_force_per_count * (counts - self.force_zero_count)
    }

    pub fn force_from_kg(self: &WinchCalibration, kg: f32) -> f32 {
        (kg / self.kg_force_per_count) + self.force_zero_count
    }

    pub fn dist_to_m(self: &WinchCalibration, counts: f32) -> f32 {
        self.m_dist_per_count * counts
    }

    pub fn dist_from_m(self: &WinchCalibration, m: f32) -> f32 {
        m / self.m_dist_per_count
    }

    pub fn pwm_gain_from_m(self: &WinchCalibration, gain: f32) -> f32 {
        // PWM gains are motor power difference per distance/acceleration/speed.
        // Since we have meters in the denominator, it's the inverse of dist_from_m.
        gain * self.m_dist_per_count
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WinchLightingScheme {
    pub normal_color: Vector3<f32>,
    pub manual_selected_color: Vector3<f32>,
    pub manual_deselected_color: Vector3<f32>,
    pub halt_color: Vector3<f32>,
    pub error_color: Vector3<f32>,
    pub stuck_color: Vector3<f32>,
    pub command_color: Vector3<f32>,
    pub motion_color: Vector3<f32>,
    pub wavelength_m: f32,
    pub wave_window_length_m: f32,
    pub wave_amplitude: f32,
    pub wave_exponent: f32,
    pub speed_for_full_wave_amplitude_m_per_sec: f32,
    pub velocity_filter_param: f32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct LightingScheme {
    pub brightness: f32,
    pub flash_rate_hz: f32,
    pub flash_exponent: f32,
    pub winch: WinchLightingScheme,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct LightingConfig {
    pub animation: LightAnimatorConfig,
    pub current: LightingScheme,
    pub saved: BTreeMap<String, LightingScheme>,
    pub schedule: BTreeMap<NaiveTime, String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct LightAnimatorConfig {
    pub frame_rate: f32,
    pub filter_param: f32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct OverlayConfig {
    pub halt_color: Vector4<f32>,
    pub border_thickness: f32,
    pub debug_color: Vector4<f32>,
    pub debug_background_color: Vector4<f32>,
    pub debug_text_height: f32,
    pub detector_outline_min_prob: f32,
    pub detector_outline_max_thickness: f32,
    pub detector_label_min_prob: f32,
    pub detector_label_prob_values: bool,
    pub detector_default_outline_color: Vector4<f32>,
    pub label_color: Vector4<f32>,
    pub label_text_size: f32,
    pub label_background_color: Vector4<f32>,
    pub tracked_region_default_color: Vector4<f32>,
    pub tracked_region_manual_color: Vector4<f32>,
    pub tracked_region_outline_thickness: f32,
    pub gain_region_color: Vector4<f32>,
    pub particle_color: Vector4<f32>,
    pub particle_size: f32,
    pub particle_sprites: Vec<Vector4<i32>>,
    pub particle_count: usize,
    pub particle_damping: f32,
    pub particle_edge_gain: f32,
    pub particle_perpendicular_gain: f32,
    pub particle_min_distance: f32,
    pub particle_min_distance_gain: f32,
    pub particle_random_gain: f32,
    pub gimbal_rect_center: Vector2<f32>,
    pub gimbal_rect_width: f32,
    pub gimbal_background_color: Vector4<f32>,
    pub gimbal_outline_color: Vector4<f32>,
    pub gimbal_outline_thickness: f32,
    pub gimbal_cursor_color: Vector4<f32>,
    pub gimbal_cursor_size: f32,
    pub gimbal_cursor_sprite: Vector4<i32>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct VisionConfig {
    pub border_rect: Vector4<f32>,
    pub manual_control_deadzone: f32,
    pub manual_control_speed: f32,
    pub manual_control_restoring_force: f32,
    pub manual_control_restoring_force_width: f32,
    pub manual_control_timeout_sec: f32,
    pub tracking_default_area: f32,
    pub tracking_min_area: f32,
    pub tracking_max_area: f32,
    pub snap_tracked_region_to: Vec<(String, f32)>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct GimbalConfig {
    pub max_rate: f32,
    pub yaw_gains: Vec<GimbalTrackingGain>,
    pub pitch_gains: Vec<GimbalTrackingGain>,
    pub yaw_limits: (i16, i16),
    pub pitch_limits: (i16, i16),
    pub limiter_gain: f32,
    pub limiter_slowdown_extent: Vector2<f32>,
    pub hold_p_gain: f32,
    pub hold_i_gain: f32,
    pub tracking_i_decay_rate: f32,
    pub hold_i_decay_rate: f32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct GimbalTrackingGain {
    pub width: f32,
    pub p_gain: f32,
    pub i_gain: f32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BotParams {
    pub gimbal_max_control_rate: f32,
    pub manual_control_velocity_m_per_sec: f32,
    pub accel_limit_m_per_sec2: f32,
    pub force_neg_motion_min_kg: f32,
    pub force_pos_motion_max_kg: f32,
    pub force_lockout_below_kg: f32,
    pub force_lockout_above_kg: f32,
    pub force_filter_param: f32,
    pub force_return_velocity_max_m_per_sec: f32,
    pub deadband_position_err_m: f32,
    pub deadband_velocity_limit_m_per_sec: f32,
    pub pos_err_filter_param: f32,
    pub vel_err_filter_param: f32,
    pub integral_err_decay_param: f32,
    pub pwm_gain_p: f32,
    pub pwm_gain_i: f32,
    pub pwm_gain_d: f32,
    pub pwm_bias: f32,
    pub pwm_minimum: f32,
    pub pwm_hz_low_motion: f32,
    pub pwm_hz_high_motion: f32,
    pub pwm_hz_filter_param: f32,
    pub pwm_velocity_threshold: f32,
    pub winch_watchdog_millis: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct MetricsConfig {
    pub influxdb_host: String,
    pub database: String,
    pub batch_size: usize,
    pub max_sample_hz: f32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WebConfig {
    pub http_addr: SocketAddr,
    pub ws_addr: SocketAddr,
    pub web_root_path: String,
    pub connection_file_path: String,
    pub open_browser: bool,
    pub browser_port_override: Option<u16>,
}

fn all_if_addr() -> IpAddr {
    // Bind to all interfaces; we need at least localhost and the LAN
    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
}

impl WebConfig {
    pub fn http_bind_addr(self: &WebConfig) -> SocketAddr {
        SocketAddr::new(all_if_addr(), self.http_addr.port())
    }

    pub fn ws_bind_addr(self: &WebConfig) -> SocketAddr {
        SocketAddr::new(all_if_addr(), self.ws_addr.port())
    }

    pub fn http_uri(self: &WebConfig, secret_key: &str, custom_port: Option<u16>) -> String {
        let mut http_addr = self.http_addr.clone();
        if let Some(port) = custom_port {
            http_addr.set_port(port);
        }
        format!("http://{}/#?k={}", http_addr, secret_key)
    }

    pub fn ws_uri(self: &WebConfig) -> String {
        format!("ws://{}", self.ws_addr)
    }
}

impl Config {
    pub fn merge(self: &Config, updates: Value) -> Result<Config, serde_json::Error> {
        let mut value = to_value(self)?;
        merge_values(&mut value, updates);
        from_value(value)
    }
}

impl SharedConfigFile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<SharedConfigFile, String> {
        let path = path.as_ref().to_path_buf();
        let mut file = err_string(File::open(&path))?;
        let mut buffer = String::new();
        err_string(file.read_to_string(&mut buffer))?;
        let config = Arc::new(Mutex::new(err_string(serde_yaml::from_str(&buffer))?));
        let (async_save_channel, save_thread_receiver) = channel();
        SharedConfigFile::start_save_thread(path, save_thread_receiver);
        Ok(SharedConfigFile { config, async_save_channel })
    }

    pub fn get_latest(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    pub fn set(&self, config: Config) {
        *self.config.lock().unwrap() = config.clone();
        drop(self.async_save_channel.send(config));
    }

    fn start_save_thread(path: PathBuf, receiver: Receiver<Config>) {
        const CONSOLIDATION_MILLIS : u64 = 1000;
        thread::Builder::new().name("SharedConfigFile".into()).spawn(move || {
            loop {
                // Block until any config save at all shows up
                let config = match receiver.recv() {
                    Ok(config) => config,
                    Err(_) => return,
                };

                // Wait a bit to see if something newer shows up
                thread::sleep(Duration::from_millis(CONSOLIDATION_MILLIS));
                let config = match receiver.try_iter().last() {
                    Some(config) => config,
                    None => config,
                };

                let string = serde_yaml::to_string(&config).unwrap();
                let af = atomicwrites::AtomicFile::new(&path, atomicwrites::AllowOverwrite);
                af.write( |f| {
                    f.write_all(string.as_bytes())
                }).expect("Failed to write new configuration file");
            }
        }).unwrap();
    }
}

fn merge_values(base: &mut Value, updates: Value) {
    match updates {
        Value::Array(update_arr) => {
            if let Value::Array(ref mut base_arr) = *base {
                for (i, item) in update_arr.into_iter().enumerate() {
                    match item {
                        // Nulls in an array are used to skip that element
                        Value::Null => {},
                        item => {
                            while i >= base_arr.len() {
                                base_arr.push(Value::Null);
                            }
                            merge_values(&mut base_arr[i], item);
                        }
                    }
                }
            } else {
                *base = Value::Array(update_arr);
            }
        },
        Value::Object(update_obj) => {
            if let Value::Object(ref mut base_obj) = *base {
                for (key, item) in update_obj.into_iter() {
                    match item {
                        // Nulls in an object will *delete* that element.
                        // This is used to remove elements in the lighting scheme map.
                        Value::Null => {
                            base_obj.remove(&key);
                        },
                        item => {
                            if let Some(value) = base_obj.get_mut(&key) {
                                merge_values(value, item);
                                continue;
                            }
                            base_obj.insert(key, item);
                        }
                    }
                }
            } else {
                *base = Value::Object(update_obj);
            }
        },
        update => {
            *base = update;
        }
    }
}

fn err_string<T, U: Display>(result: Result<T, U>) -> Result<T, String> {
    result.map_err(|err| format!("{}", err))
}
