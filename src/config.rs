//! Bot configuration

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::io;
use std::fmt::Display;
use serde_json;
use serde_json::{Value, from_value, to_value};
use serde_yaml;
use atomicwrites;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc::{Sender, Receiver, channel};
use websocket;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Config {
    pub mode: ControllerMode,
    pub controller_addr: SocketAddr,
    pub flyer_addr: SocketAddr,
    pub web: WebConfig,
    pub params: BotParams,
    pub winches: Vec<WinchConfig>,
}

pub struct ConfigFile {
    pub path: PathBuf,
    pub config: Config,
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
    pub loc: Point3,
    pub calibration: WinchCalibration,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    pub fn new(x: f64, y: f64, z: f64) -> Point3 {
        Point3 { x, y, z }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WinchCalibration {
    pub force_zero_count: f64,
    pub kg_force_per_count: f64,
    pub m_dist_per_count: f64,
}

impl WinchCalibration {
    pub fn force_to_kg(self: &WinchCalibration, counts: f64) -> f64 {
        self.kg_force_per_count * (counts - self.force_zero_count)
    }

    pub fn force_from_kg(self: &WinchCalibration, kg: f64) -> f64 {
        (kg / self.kg_force_per_count) + self.force_zero_count
    }

    pub fn dist_to_m(self: &WinchCalibration, counts: f64) -> f64 {
        self.m_dist_per_count * counts
    }

    pub fn dist_from_m(self: &WinchCalibration, m: f64) -> f64 {
        m / self.m_dist_per_count
    }

    pub fn pwm_gain_from_m(self: &WinchCalibration, gain: f64) -> f64 {
        // PWM gains are motor power difference per distance/acceleration/speed.
        // Since we have meters in the denominator, it's the inverse of dist_from_m.
        gain * self.m_dist_per_count
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BotParams {
    pub accel_rate_m_per_sec2: f64,
    pub manual_control_velocity_m_per_sec: f64,
    pub force_min_kg: f64,
    pub force_max_kg: f64,
    pub force_filter_param: f64,
    pub diff_filter_param: f64,
    pub pwm_gain_p: f64,
    pub pwm_gain_i: f64,
    pub pwm_gain_d: f64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WebConfig {
    pub http_addr: SocketAddr,
    pub ws_addr: SocketAddr,
    pub web_root_path: String,
    pub connection_file_path: String,
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

    pub fn http_uri(self: &WebConfig, secret_key: &str) -> String {
        format!("http://{}/#?k={}", self.http_addr, secret_key)
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

impl ConfigFile {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<ConfigFile, String> {
        let path = path.as_ref().to_path_buf();
        let mut file = err_string(File::open(&path))?;
        let mut buffer = String::new();
        err_string(file.read_to_string(&mut buffer))?;
        let config = err_string(serde_yaml::from_str(&buffer))?;
        let (async_save_channel, save_thread_receiver) = channel();
        ConfigFile::start_save_thread(path.clone(), save_thread_receiver);
        Ok(ConfigFile { path, config, async_save_channel })
    }

    pub fn save_async(self: &ConfigFile) {
        drop(self.async_save_channel.send(self.config.clone()));
    }

    pub fn start_save_thread(path: PathBuf, receiver: Receiver<Config>) {
        const CONSOLIDATION_MILLIS : u64 = 1000;
        thread::spawn(move || {
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

                println!("Saving configuration");
                let string = serde_yaml::to_string(&config).unwrap();
                let af = atomicwrites::AtomicFile::new(&path, atomicwrites::AllowOverwrite);
                af.write( |f| {
                    f.write_all(string.as_bytes())
                }).expect("Failed to write new configuration file");
            }
        });
    }
}

fn merge_values(base: &mut Value, updates: Value) {
    match updates {
        Value::Array(update_arr) => {
            if let Value::Array(ref mut base_arr) = *base {
                for (i, item) in update_arr.into_iter().enumerate() {
                    match item {
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
                    if let Some(mut value) = base_obj.get_mut(&key) {
                        merge_values(value, item);
                        continue;
                    }
                    base_obj.insert(key, item);
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
