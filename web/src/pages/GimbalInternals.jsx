import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import reactCSS from 'reactcss';
import { Chart, Series } from '../BotChart';

let descriptions = {
    0x02: "Status flags, bit2=motor_power, bit1=usually_1",
    0x03: "Follow Loop (or Bot-Controller) → Per-joint velocity target",
    0x04: "Angular velocity from IMU at camera mount",
    0x05: "dynamic",
    0x06: "dynamic",
    0x07: "dynamic",
    0x08: "unknown, related to current state of follow loop (integrated heading)",
    0x09: "Integrated IMU angles, 1/100 degree",
    0x18: "Motor control mode (for calibration maybe?), (0-5)",
    0x28: "motor phase offset estimate, eq ~16000 (param27?) at CAL1",
    0x2c: "Magnetic encoder positions per joint",
    0x47: "motor unknown, FIR filter",
    0x48: "motor unknown, FIR filter",
    0x4a: "dynamic, quantized velocity related",
    0x4b: "Follow loop, offset angle for param08",
    0x4c: "Follow loop, angle error",
    0x4d: "Center calibration point 0",
    0x53: "Joystick state flag (t=2)",
    0x59: "Unknown IMU calibration point (roll, t=2)",
    0x5a: "Unknown IMU calibration point (pitch, t=2)",
    0x5b: "Unknown IMU calibration point (yaw?, t=2)",
    0x5c: "Attitude correction (roll, t=2)",
    0x5d: "Attitude correction (pitch, t=2)",
    0x5e: "Attitude correction (yaw?, t=2)",
    0x63: "Follow loop, enable/disable flag (t=0)",
    0x64: "Off-center calibration point 1",
    0x65: "Follow loop, user-settable rate parameter",
    0x66: "Joystick mode flag",
    0x67: "unknown, reset by windows software prior to motor power-on",
    0x68: "Follow loop, joystick offset (t=2)",
    0x69: "IMU type code (t=2)",
    0x6f: "motor unknown, unfiltered, position-like",
    0x70: "motor unknown, unfiltered, velocity-like",
    0x7f: "Firmware version",
};

export default (props) => {
	let params = [];
	for (let i = 0; i < 128; i++) {
        params.push(<GimbalRow index={i} key={`row-${i}`}/>);
	}

    let request_all = [];
    for (let index = 0; index < 0x80; index++) {
        for (let target = 0; target < 3; target++) {
            request_all.push({
                addr: { index, target },
                scope: "Once"
            });
        }
    }

    return <div>

        <h4>Polling</h4>

        <GimbalPollerToggle block interval={1000.0} requests={request_all}> All parameters (1 sec) </GimbalPollerToggle>
        <GimbalPollerToggle block interval={10000.0} requests={request_all}> All parameters (10 sec) </GimbalPollerToggle>

        <h4>Firmware parameters</h4>
        <div> { params } </div>

    </div>;
}


class GimbalRow extends Component {
    constructor() {
        super();
        this.state = {
            graphEnabled: false,
            editEnabled: false,
        };
    }

    onChangeEditCheckbox(event) {
        this.setState({
            editEnabled: event.target.checked
        });
    }

    render() {
        let { index } = this.props;

        let row = [];
        for (let t = 0; t < 3; t++) {
            row.push(<GimbalParam index={index} target={t} key={`target-${t}`} />);
        }

        let requests_for_scope = (scope) => {
            let requests = [];
            for (let target = 0; target < 3; target++) {
                requests.push({
                    addr: { index, target },
                    scope
                });
            }
            return requests;
        };
        let req_once = requests_for_scope("Once");
        let req_continuous = requests_for_scope("Continuous");

        return <div className="GimbalRow">
            <span className="index">{ ("00" + index.toString(16)).slice(-2) }</span>
            {row}
            <GimbalPollerToggle interval={300.0} requests={req_continuous}>cont</GimbalPollerToggle>
            <GimbalPollerToggle interval={100.0} requests={req_once}>100ms</GimbalPollerToggle>
            <GimbalPollerToggle interval={1000.0} requests={req_once}>1s</GimbalPollerToggle>
            <span className="GimbalPollerToggle">
                <input type="checkbox" checked={this.state.graphEnabled} onChange={(event) => {
                    this.setState({ graphEnabled: event.target.checked });
                }} />
                <label>graph</label>
            </span>
            <span className="GimbalPollerToggle">
                <input type="checkbox" checked={this.state.editEnabled} onChange={(event) => {
                    this.setState({ editEnabled: event.target.checked });
                }} />
                <label>edit</label>
            </span>
            <span className="description">
                { descriptions[index] || "" }
            </span>
            { this.state.editEnabled && <div>
                <GimbalSlider index={index} target={0} />
                <GimbalSlider index={index} target={1} />
                <GimbalSlider index={index} target={2} />
            </div> }
            { this.state.graphEnabled && <div>
                <Chart>
                    <Series
                        strokeStyle='#a22'
                        value={ (model) => model.gimbal_values[index][0].message.GimbalValue[0].value }
                        trigger={ (model) => model.gimbal_values[index][0].local_timestamp }
                        timestamp={ (model) => model.gimbal_values[index][0].local_timestamp } />
                </Chart>
                <Chart>
                    <Series
                        strokeStyle='#2a2'
                        value={ (model) => model.gimbal_values[index][1].message.GimbalValue[0].value }
                        trigger={ (model) => model.gimbal_values[index][1].local_timestamp }
                        timestamp={ (model) => model.gimbal_values[index][1].local_timestamp } />
                </Chart>
                <Chart>
                    <Series
                        strokeStyle='#22a'
                        value={ (model) => model.gimbal_values[index][2].message.GimbalValue[0].value }
                        trigger={ (model) => model.gimbal_values[index][2].local_timestamp }
                        timestamp={ (model) => model.gimbal_values[index][2].local_timestamp } />
                </Chart>
            </div> }
        </div>;
    }
}

class GimbalPollerToggle extends Component {

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    constructor() {
        super();
        this.state = {
            enabled: false,
        };
        this.timeout = null;
        this.setTimer(0);
    }

    render() {
        let s = <span className="GimbalPollerToggle">
            <input type="checkbox" onChange={this.onChange.bind(this)} checked={this.state.enabled} />
            <label>{this.props.children}</label>
        </span>;
        if (this.props.block) {
            return <div className="GimbalPollerToggle"> {s} </div>;
        } else {
            return s;
        }
    }

    onChange(event) {
        this.setState({
            enabled: event.target.checked
        });
    }

    componentDidUpdate() {
        this.setTimer();
    }

    componentWillUnmount() {
        this.clearTimer();
    }

    clearTimer() {
        if (this.timeout != null) {
            window.clearTimeout(this.timeout);
            this.timeout = null;
        }
    }

    setTimer(interval) {
        this.clearTimer();
        if (this.state.enabled) {
            this.timeout = window.setTimeout(this.onTimeout.bind(this), interval);
        }
    }

    onTimeout() {
        this.context.botConnection.send({
            Command: {
                GimbalValueRequests: this.props.requests
            }
        });
        this.setTimer(this.props.interval);
    }
}

class GimbalParam extends Component {

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

 	constructor() {
        super();
        this.state = {
        	opacity: 0.0,
            op: "unread",
            dyn: "static",
            value: 0,
        };
 	}

    render() {
        let styles = reactCSS({
	        'default': {
	            current: {
	            	opacity: this.state.opacity
	            }
	        }
	    });
        return <span className={`GimbalParam op-${this.state.op} ${this.state.dyn}`}>
        	<span style={styles.current}>{this.state.value}</span>
        </span>;
    }

    componentDidMount() {
        this.context.botConnection.events.on('frame', this.handleFrame);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('frame', this.handleFrame);
    }

    handleFrame = (model) => {
    	const tsm = (model.gimbal_values[this.props.index] || [])[this.props.target];
    	if (tsm) {
    		const fade_duration = 500;
    		const age = ((new Date()).getTime() - tsm.local_timestamp);
    		const opacity = Math.max(0.5, 1.0 - age / fade_duration);
    		const value = tsm.message.GimbalValue[0].value;
            const op = tsm.message.GimbalValue[1];
            const dyn = (this.state.op !== "unread" && this.state.value !== value) ? "dynamic" : this.state.dyn;
    		this.setState({ opacity, op, value, dyn });
    	}
    }
}

class GimbalSlider extends Component {
    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    constructor() {
        super();
        this.state = {
            value: 0,
        };
    }

    render() {
        let { index, target, ...props } = this.props;
        return (
            <div className="GimbalSlider">
                <input
                    min="-32767"
                    max="32767"
                    step="1"
                    {...props}
                    type="range"
                    value={this.state.value}
                    onChange={this.handleChange.bind(this)}
                />
                <span>
                    {this.state.value}
                </span>
            </div>
        );
    }

    componentDidMount() {
        this.context.botConnection.events.on('frame', this.handleFrame);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('frame', this.handleFrame);
    }

    handleFrame = (model) => {
        const tsm = (model.gimbal_values[this.props.index] || [])[this.props.target];
        if (tsm) {
            const value = tsm.message.GimbalValue[0].value;
            this.setState({ value });
        }
    }

    handleChange(event) {
        const value = parseInt(event.target.value, 0);
        this.setState({ value });
        this.context.botConnection.socket.send(JSON.stringify({
            Command: {
                GimbalValueWrite: {
                    addr: {
                        target: this.props.target,
                        index: this.props.index,
                    },
                    value
                }
            }
        }));
    }
}