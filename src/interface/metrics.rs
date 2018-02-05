use controller::ControllerPort;
use config::{Config, SharedConfigFile};
use message::{TimestampedMessage, Message};
use std::thread;
use std::net::UdpSocket;
use cadence::{MetricSink, BufferedUdpMetricSink};


pub fn start(config: &SharedConfigFile, controller: &ControllerPort) {
    let config = config.get_latest();
    if let Some(statsd_addr) = config.metrics.statsd_addr {

        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let sink = BufferedUdpMetricSink::from(statsd_addr, socket).expect("Failed to connect stats");
        let mut bus_receiver = controller.add_rx();

        thread::Builder::new().name("Metrics".into()).spawn(move || {
            loop {
                let msg = bus_receiver.recv().unwrap();
                message_metrics(&config, &sink, &msg);
            }
        }).unwrap();
    }
}

fn message_metrics(config: &Config, sink: &BufferedUdpMetricSink, tsm: &TimestampedMessage) {

    if let &Message::WinchStatus(id, ref status) = &tsm.message {
        let cal = &config.winches[id].calibration;

        // FIX ME: Telegraf spends 5% CPU just consolidating this stuff.. we should downsample it in-process.

        sink.emit(&format!("{}.winch.sensors.force,winch_id={}:{}|g",
            config.metrics.prefix, id,
            cal.force_to_kg(status.sensors.force.filtered)
        )).unwrap();

        sink.emit(&format!("{}.winch.motor.pwm,winch_id={}:{}|g",
            config.metrics.prefix, id,
            status.motor.pwm.total
        )).unwrap();
    }
}
