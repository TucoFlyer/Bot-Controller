use controller::ControllerPort;
use config::{Config, SharedConfigFile, ControllerMode};
use message::{TimestampedMessage, Message, Command, ManualControlAxis};
use std::thread;
use std::sync::mpsc::sync_channel;
use std::mem;
use std::env;
use std::collections::HashMap;
use num::range;
use std::time::{Duration, Instant};
use serde_json::Map;
use serde_json::Value as JsonValue;
use serde_json;
use influx_db_client::{Point, Value};
use influx_db_client;
use chrono::{DateTime, Utc};

struct MetricSampler {
    sync: TimeSync,
    min_interval: Duration,
    winch_ts: Vec<Instant>,
    flyer_ts: Instant,
    flush_ts: Instant,
    gimbal_control_ts: Instant,
    object_detector_ts: Instant,
    region_tracker_ts: Instant,
    manual_axes: HashMap<ManualControlAxis, f32>,
    message_counts: HashMap<&'static str, u64>,
}

impl MetricSampler {
    fn new(max_sample_hz: f32, num_winches: usize) -> MetricSampler {
        let now = Instant::now();
        MetricSampler {
            sync: TimeSync::new(),
            min_interval: Duration::new(0, (1e9 / max_sample_hz) as u32),
            winch_ts: range(0, num_winches).map(|_| now).collect(),
            flyer_ts: now,
            object_detector_ts: now,
            region_tracker_ts: now,
            flush_ts: now,
            gimbal_control_ts: now,
            manual_axes: HashMap::new(),
            message_counts: HashMap::new(),
        }
    }

    fn handle_message(&mut self, points: &mut Vec<Point>, config: &mut Config, tsm: &TimestampedMessage) {
        *self.message_counts.entry("message").or_insert(0) += 1;
        match &tsm.message {

            &Message::WinchStatus(id, ref status) => {
                *self.message_counts.entry("winch_status").or_insert(0) += 1;
                if tsm.timestamp >= self.winch_ts[id] + self.min_interval {
                    self.winch_ts[id] = tsm.timestamp;
                    let cal = &config.winches[id].calibration;

                    let mut p = Point::new("winch.status");
                    p.add_timestamp(self.sync.to_millis(tsm.timestamp));
                    p.add_tag("id", Value::Integer(id as i64));
                    p.add_field("force", Value::Float(cal.force_to_kg(status.sensors.force.filtered).into()));
                    p.add_field("velocity", Value::Float(cal.dist_to_m(status.sensors.velocity).into()));
                    p.add_field("position", Value::Float(cal.dist_to_m(status.sensors.position as f32).into()));
                    p.add_field("position_err", Value::Float(cal.dist_to_m(status.motor.position_err as f32).into()));
                    p.add_field("position_err.filtered", Value::Float(cal.dist_to_m(status.motor.pos_err_filtered).into()));
                    p.add_field("position_err.integral", Value::Float(cal.dist_to_m(status.motor.pos_err_integral).into()));
                    p.add_field("velocity_err", Value::Float(cal.dist_to_m(status.motor.vel_err_inst).into()));
                    p.add_field("velocity_err.filtered", Value::Float(cal.dist_to_m(status.motor.vel_err_filtered).into()));
                    p.add_field("pwm.p", Value::Float(status.motor.pwm.p.into()));
                    p.add_field("pwm.i", Value::Float(status.motor.pwm.i.into()));
                    p.add_field("pwm.d", Value::Float(status.motor.pwm.d.into()));
                    p.add_field("pwm.total", Value::Float(status.motor.pwm.total.into()));
                    p.add_field("pwm.hz", Value::Float(status.motor.pwm.hz.into()));
                    p.add_field("pwm.enabled", Value::Boolean(status.motor.pwm.enabled != 0));
                    points.push(p);
                }
            },

            &Message::FlyerSensors(ref status) => {
                *self.message_counts.entry("flyer_sensors").or_insert(0) += 1;
                if tsm.timestamp >= self.flyer_ts + self.min_interval {
                    self.flyer_ts = tsm.timestamp;

                    let mut p = Point::new("flyer.sensors");
                    p.add_timestamp(self.sync.to_millis(tsm.timestamp));

                    p.add_field("xband.speed", Value::Integer(status.xband.speed_measure.into()));
                    for (i, y) in status.lidar.ranges.iter().enumerate() {
                        p.add_field(format!("lidar.{}", i), Value::Integer(*y as i64));
                    }
                    for (i, y) in status.analog.values.iter().enumerate() {
                        p.add_field(format!("analog.{}", i), Value::Integer(*y as i64));
                    }
                    p.add_field("temperature", Value::Integer(status.imu.temperature.into()));
                    p.add_field("accelerometer.x", Value::Integer(status.imu.accelerometer[0].into()));
                    p.add_field("accelerometer.y", Value::Integer(status.imu.accelerometer[1].into()));
                    p.add_field("accelerometer.z", Value::Integer(status.imu.accelerometer[2].into()));
                    p.add_field("magnetometer.x", Value::Integer(status.imu.magnetometer[0].into()));
                    p.add_field("magnetometer.y", Value::Integer(status.imu.magnetometer[1].into()));
                    p.add_field("magnetometer.z", Value::Integer(status.imu.magnetometer[2].into()));
                    p.add_field("gyroscope.x", Value::Integer(status.imu.gyroscope[0].into()));
                    p.add_field("gyroscope.y", Value::Integer(status.imu.gyroscope[1].into()));
                    p.add_field("gyroscope.z", Value::Integer(status.imu.gyroscope[2].into()));
                    points.push(p);
                }
            },

            &Message::GimbalControlStatus(ref status) => {
                *self.message_counts.entry("gimbal_control_status").or_insert(0) += 1;
                if tsm.timestamp >= self.gimbal_control_ts + self.min_interval {
                    self.gimbal_control_ts = tsm.timestamp;

                    let mut p = Point::new("gimbal.control.status");
                    p.add_timestamp(self.sync.to_millis(tsm.timestamp));

                    p.add_field("angle.x", Value::Integer(status.angles[0].into()));
                    p.add_field("angle.y", Value::Integer(status.angles[1].into()));
                    p.add_field("rate.x", Value::Integer(status.rates[0].into()));
                    p.add_field("rate.y", Value::Integer(status.rates[1].into()));
                    p.add_field("motor_power.0", Value::Boolean(status.motor_power[0]));
                    p.add_field("motor_power.1", Value::Boolean(status.motor_power[1]));
                    p.add_field("motor_power.2", Value::Boolean(status.motor_power[2]));
                    p.add_field("current.0", Value::Float(status.current[0].into()));
                    p.add_field("current.1", Value::Float(status.current[1].into()));
                    p.add_field("current.2", Value::Float(status.current[2].into()));
                    p.add_field("current_osc_detector.0", Value::Float(status.current_osc_detector[0].into()));
                    p.add_field("current_osc_detector.1", Value::Float(status.current_osc_detector[1].into()));
                    p.add_field("current_osc_detector.2", Value::Float(status.current_osc_detector[2].into()));
                    p.add_field("current_peak_detector.0", Value::Float(status.current_peak_detector[0].into()));
                    p.add_field("current_peak_detector.1", Value::Float(status.current_peak_detector[1].into()));
                    p.add_field("current_peak_detector.2", Value::Float(status.current_peak_detector[2].into()));
                    p.add_field("current_error_duration", Value::Float(status.current_error_duration.into()));
                    p.add_field("supply_voltage", Value::Float(status.supply_voltage.into()));
                    points.push(p);
                }
            },

            &Message::ConfigIsCurrent(ref new_config) => {
                *self.message_counts.entry("config_is_current").or_insert(0) += 1;
                *config = new_config.clone();
                let json_config = serde_json::to_value(new_config).unwrap();
                let mut p = Point::new("config");
                p.add_timestamp(self.sync.to_millis(tsm.timestamp));

                if let JsonValue::Object(map) = json_config {
                    add_json_object_to_point(&mut p, None, map);
                }

                let modes = [
                    ("mode.halted", &ControllerMode::Halted),
                    ("mode.normal", &ControllerMode::Normal),
                    ("mode.manual_flyer", &ControllerMode::ManualFlyer),
                    ("mode.manual_winch", &ControllerMode::ManualWinch(0)),
                ];
                for &(name, value) in modes.iter() {
                    let is_current = mem::discriminant(&config.mode) == mem::discriminant(value);
                    p.add_field(name, Value::Boolean(is_current));
                }

                points.push(p);
            },

            &Message::PublicInput(_) => {
                *self.message_counts.entry("public_input").or_insert(0) += 1;
            },

            &Message::UpdateConfig(_) => {
                *self.message_counts.entry("update_config").or_insert(0) += 1;
            },

            &Message::GimbalValue(_, _) => {
                *self.message_counts.entry("gimbal_value").or_insert(0) += 1;
            },

            &Message::UnhandledGimbalPacket(_) => {
                *self.message_counts.entry("unhandled_gimbal_packet").or_insert(0) += 1;
            },

            &Message::CameraOverlayScene(ref rects) => {
                *self.message_counts.entry("camera_overlay_scene").or_insert(0) += 1;
                *self.message_counts.entry("camera_overlay_scene.rects").or_insert(0) += rects.len() as u64;
            },

            &Message::CameraInitTrackedRegion(_) => {
                *self.message_counts.entry("camera_init_tracked_region").or_insert(0) += 1;
            },

            &Message::Command(ref cmd) => {
                *self.message_counts.entry("command").or_insert(0) += 1;
                match cmd {

                    &Command::ManualControlValue(ref axis, value) => {
                       *self.message_counts.entry("manual_control_value").or_insert(0) += 1;
                       self.manual_axes.insert(axis.clone(), value);
                    },

                    &Command::ManualControlReset => {
                       *self.message_counts.entry("manual_control_reset").or_insert(0) += 1;
                        self.manual_axes.clear();
                    },

                    &Command::CameraObjectDetection(ref v) => {
                       *self.message_counts.entry("camera_object_detection").or_insert(0) += 1;
                        if tsm.timestamp >= self.object_detector_ts + self.min_interval {
                            self.object_detector_ts = tsm.timestamp;
                            let mut p = Point::new("camera.object_detector");
                            p.add_timestamp(self.sync.to_millis(tsm.timestamp));
                            p.add_field("detector_nsec", Value::Integer(v.detector_nsec as i64));
                            p.add_field("object_count", Value::Integer(v.objects.len() as i64));
                            points.push(p);
                        }
                    },

                    &Command::CameraRegionTracking(ref v) => {
                       *self.message_counts.entry("camera_region_tracking").or_insert(0) += 1;
                       if tsm.timestamp >= self.region_tracker_ts + self.min_interval {
                            self.region_tracker_ts = tsm.timestamp;
                            let mut p = Point::new("camera.region_tracker");
                            p.add_timestamp(self.sync.to_millis(tsm.timestamp));
                            p.add_field("tracker_nsec", Value::Integer(v.tracker_nsec as i64));
                            p.add_field("psr", Value::Float(v.psr.into()));
                            p.add_field("age", Value::Integer(v.age as i64));
                            points.push(p);
                        }
                    },

                    &Command::SetMode(_) => {
                       *self.message_counts.entry("set_mode").or_insert(0) += 1;
                    },

                    &Command::GimbalMotorEnable(_) => {
                       *self.message_counts.entry("gimbal_motor_enable").or_insert(0) += 1;
                    },

                    &Command::GimbalPacket(_) => {
                       *self.message_counts.entry("gimbal_packet").or_insert(0) += 1;
                    },

                    &Command::GimbalValueWrite(_) => {
                       *self.message_counts.entry("gimbal_value_write").or_insert(0) += 1;
                    },

                    &Command::GimbalValueRequests(_) => {
                       *self.message_counts.entry("gimbal_value_requests").or_insert(0) += 1;
                    },
                }
            }
        }

        // Poll for periodic flush of stats we aggregate here
        if tsm.timestamp >= self.flush_ts + self.min_interval {
            self.flush_ts = tsm.timestamp;
            let timestamp = self.sync.to_millis(tsm.timestamp);

            let mut p = Point::new("manual_controls");
            p.add_timestamp(timestamp);
            p.add_field("camera.yaw", Value::Float(*self.manual_axes.entry(ManualControlAxis::CameraYaw).or_insert(0.0) as f64));
            p.add_field("camera.pitch", Value::Float(*self.manual_axes.entry(ManualControlAxis::CameraPitch).or_insert(0.0) as f64));
            p.add_field("relative.x", Value::Float(*self.manual_axes.entry(ManualControlAxis::RelativeX).or_insert(0.0) as f64));
            p.add_field("relative.y", Value::Float(*self.manual_axes.entry(ManualControlAxis::RelativeY).or_insert(0.0) as f64));
            p.add_field("relative.z", Value::Float(*self.manual_axes.entry(ManualControlAxis::RelativeZ).or_insert(0.0) as f64));
            points.push(p);

            let mut p = Point::new("message_counts");
            p.add_timestamp(timestamp);
            for (name, count) in self.message_counts.iter() {
                p.add_field(name, Value::Integer(*count as i64));
            }
            points.push(p);
        }
    }
}

fn add_json_field_to_point(p: &mut Point, name: &str, val: JsonValue) {
    match val {
        JsonValue::Null => p.add_field(name, Value::String("null".into())),
        JsonValue::Bool(b) => p.add_field(name, Value::Boolean(b)),
        JsonValue::String(s) => p.add_field(name, Value::String(s)),
        JsonValue::Number(n) => if let Some(ival) = n.as_i64() {
            p.add_field(name, Value::Integer(ival));
        } else if let Some(fval) = n.as_f64() {
            p.add_field(name, Value::Float(fval));
        },
        JsonValue::Array(v) => {
            for (key, value) in v.into_iter().enumerate() {
                add_json_field_to_point(p, &format!("{}.{}", name, key), value);
            }
        },
        JsonValue::Object(m) => {
            add_json_object_to_point(p, Some(name), m);
        },
    }
}

fn add_json_object_to_point(p: &mut Point, prefix: Option<&str>, m: Map<String, JsonValue>) {
    for (key, value) in m.into_iter() {
        let name = match prefix {
            Some(prefix) => format!("{}.{}", prefix, key),
            None => key,
        };
        add_json_field_to_point(p, &name, value);
    }
}

struct TimeSync {
    instant: Instant,
    datetime: DateTime<Utc>,
}

impl TimeSync {
    fn new() -> TimeSync {
        TimeSync {
            instant: Instant::now(),
            datetime: Utc::now()
        }
    }

    fn to_millis(&self, timestamp: Instant) -> i64 {
        let millis = self.datetime.timestamp() as i64 * 1000 + self.datetime.timestamp_subsec_millis() as i64;
        millis + if timestamp > self.instant {
            let diff = timestamp - self.instant;
            diff.as_secs() as i64 * 1000 + diff.subsec_millis() as i64
        } else {
            let diff = self.instant - timestamp;
            diff.as_secs() as i64 * -1000 - diff.subsec_millis() as i64
        }
    }
}

pub fn start(config: &SharedConfigFile, controller: &ControllerPort) {
    let mut config = config.get_latest();
    if let Some(metrics_config) = config.metrics.clone() {
        let database = metrics_config.database;
        let batch_size = metrics_config.batch_size;
        let controller = controller.clone();
        let mut sampler = MetricSampler::new(metrics_config.max_sample_hz, config.winches.len());

        let client = influx_db_client::Client::new(&metrics_config.influxdb_host, &database);
        let client = if let Ok(username) = env::var("INFLUXDB_USERNAME") {
            client.set_authentication(username, env::var("INFLUXDB_PASSWORD").unwrap())
        } else {
            client
        };

        let (batch_sender, batch_receiver) = sync_channel(16);

        thread::Builder::new().name("Metrics sampler".into()).spawn(move || {
            let mut bus_receiver = controller.add_rx();
            let mut points = Vec::new();

            loop {
                let msg = bus_receiver.recv().unwrap();
                sampler.handle_message(&mut points, &mut config, &msg);

                if points.len() >= batch_size {
                    let mut batch = Vec::new();
                    mem::swap(&mut points, &mut batch);
                    if batch_sender.try_send(batch).is_err() {
                        println!("Dropping metrics, transmitter must be busy");
                    }
                }
            }
        }).unwrap();

        thread::Builder::new().name("Metrics transmitter".into()).spawn(move || {
            loop {
                let batch = batch_receiver.recv().unwrap();
                let batch = influx_db_client::Points::create_new(batch);
                let precision = Some(influx_db_client::Precision::Milliseconds);

                match client.write_points(batch, precision, None) {
                    Ok(_) => (),

                    Err(influx_db_client::Error::DataBaseDoesNotExist(_)) => {
                        client.create_database(&database).expect("Failed to create metrics database");
                    },

                    err => {
                        println!("Failed to write metrics: {:?}", err);
                    },
                };
            }
        }).unwrap();
    }
}
