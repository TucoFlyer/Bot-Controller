use controller::ControllerPort;
use config::{Config, SharedConfigFile};
use message::{TimestampedMessage, Message, Command, ManualControlAxis};
use std::thread;
use std::sync::mpsc::sync_channel;
use std::mem;
use std::collections::HashMap;
use num::range;
use std::time::{Duration, Instant};
use influx_db_client::{Client, Point, Value, Precision};
use influx_db_client;
use chrono::{DateTime, Utc};

struct MetricSampler {
    sync: TimeSync,
    min_interval: Duration,
    winch_ts: Vec<Instant>,
    flyer_ts: Instant,
    manual_ts: Instant,
    gimbal_control_ts: Instant,
    object_detector_ts: Instant,
    region_tracker_ts: Instant,
    manual_axes: HashMap<ManualControlAxis, f32>,
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
            manual_ts: now,
            gimbal_control_ts: now,
            manual_axes: HashMap::new(),
        }
    }

    fn handle_message(&mut self, points: &mut Vec<Point>, config: &mut Config, tsm: &TimestampedMessage) {
        match &tsm.message {

            &Message::WinchStatus(id, ref status) => {
                if tsm.timestamp >= self.winch_ts[id] + self.min_interval {
                    self.winch_ts[id] = tsm.timestamp;
                    let cal = &config.winches[id].calibration;

                    let mut p = Point::new("winch.status");
                    p.add_timestamp(self.sync.to_millis(tsm.timestamp));
                    p.add_tag("id", Value::Integer(id as i64));
                    p.add_field("force", Value::Float(cal.force_to_kg(status.sensors.force.filtered).into()));
                    p.add_field("velocity", Value::Float(cal.dist_to_m(status.sensors.velocity as f32).into()));
                    p.add_field("position_err", Value::Float(cal.dist_to_m(status.motor.position_err as f32).into()));
                    p.add_field("position_err.filtered", Value::Float(cal.dist_to_m(status.motor.pos_err_filtered).into()));
                    p.add_field("position_err.integral", Value::Float(cal.dist_to_m(status.motor.pos_err_integral).into()));
                    p.add_field("velocity_err", Value::Float(cal.dist_to_m(status.motor.vel_err_inst).into()));
                    p.add_field("velocity_err.filtered", Value::Float(cal.dist_to_m(status.motor.vel_err_filtered).into()));
                    p.add_field("pwm.p", Value::Float(status.motor.pwm.p.into()));
                    p.add_field("pwm.i", Value::Float(status.motor.pwm.i.into()));
                    p.add_field("pwm.d", Value::Float(status.motor.pwm.d.into()));
                    p.add_field("pwm.total", Value::Float(status.motor.pwm.total.into()));
                    p.add_field("pwm.enabled", Value::Boolean(status.motor.pwm.enabled != 0));
                    points.push(p);
                }
            },

            &Message::FlyerSensors(ref status) => {
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
                if tsm.timestamp >= self.gimbal_control_ts + self.min_interval {
                    self.gimbal_control_ts = tsm.timestamp;

                    let mut p = Point::new("gimbal.control.status");
                    p.add_timestamp(self.sync.to_millis(tsm.timestamp));

                    p.add_field("angle.x", Value::Integer(status.angles[0].into()));
                    p.add_field("angle.y", Value::Integer(status.angles[1].into()));
                    p.add_field("rate.x", Value::Integer(status.rates[0].into()));
                    p.add_field("rate.y", Value::Integer(status.rates[1].into()));
                    points.push(p);
                }
            },

            &Message::ConfigIsCurrent(ref new_config) => {
                *config = new_config.clone();
                let mut p = Point::new("config");
                p.add_timestamp(self.sync.to_millis(tsm.timestamp));
                p.add_field("mode", Value::String(format!("{:?}", config.mode)));
                points.push(p);
            },

            &Message::Command(Command::ManualControlValue(ref axis, value)) => {
                self.manual_axes.insert(axis.clone(), value);
            },

            &Message::Command(Command::ManualControlReset) => {
                self.manual_axes.clear();
            },

            &Message::Command(Command::CameraObjectDetection(ref v)) => {
                if tsm.timestamp >= self.object_detector_ts + self.min_interval {
                    self.object_detector_ts = tsm.timestamp;
                    let mut p = Point::new("camera.object_detector");
                    p.add_timestamp(self.sync.to_millis(tsm.timestamp));
                    p.add_field("detector_nsec", Value::Integer(v.detector_nsec as i64));
                    p.add_field("object_count", Value::Integer(v.objects.len() as i64));
                    points.push(p);
                }
            },

            &Message::Command(Command::CameraRegionTracking(ref v)) => {
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

            _ => {}
        }

        // Poll for periodic manual control updates, from the self.manual_axes hashmap
        if tsm.timestamp >= self.manual_ts + self.min_interval {
            self.manual_ts = tsm.timestamp;
            let mut p = Point::new("manual_controls");
            p.add_timestamp(self.sync.to_millis(tsm.timestamp));
            p.add_field("camera.yaw", Value::Float(*self.manual_axes.entry(ManualControlAxis::CameraYaw).or_insert(0.0) as f64));
            p.add_field("camera.pitch", Value::Float(*self.manual_axes.entry(ManualControlAxis::CameraPitch).or_insert(0.0) as f64));
            p.add_field("relative.x", Value::Float(*self.manual_axes.entry(ManualControlAxis::RelativeX).or_insert(0.0) as f64));
            p.add_field("relative.y", Value::Float(*self.manual_axes.entry(ManualControlAxis::RelativeY).or_insert(0.0) as f64));
            p.add_field("relative.z", Value::Float(*self.manual_axes.entry(ManualControlAxis::RelativeZ).or_insert(0.0) as f64));
            points.push(p);
        }
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

        let client = Client::new(&metrics_config.influxdb_host, &database);
        let client = if let Some((user, passwd)) = metrics_config.authentication {
            client.set_authentication(user, passwd)
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
                let precision = Some(Precision::Milliseconds);

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
