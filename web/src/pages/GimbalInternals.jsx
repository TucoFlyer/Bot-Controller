import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import reactCSS from 'reactcss';
import { Chart, Series } from '../BotChart';

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

        <div><GimbalPollerToggle interval={1000.0} requests={request_all}> All parameters (1 sec) </GimbalPollerToggle></div>
        <div><GimbalPollerToggle interval={10000.0} requests={request_all}> All parameters (10 sec) </GimbalPollerToggle></div>

        <h4>Firmware parameters</h4>
        <div> { params } </div>

    </div>;
}


class GimbalRow extends Component {
    constructor() {
        super();
        this.state = {
            graphEnabled: false,
        };
    }

    onChangeGraphCheckbox(event) {
        this.setState({
            graphEnabled: event.target.checked
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
            <span>
                <input type="checkbox" onChange={this.onChangeGraphCheckbox.bind(this)} checked={this.state.graphEnabled} />
                <label>graph</label>
            </span>
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
        return <span className="GimbalPollerToggle">
            <input type="checkbox" onChange={this.onChange.bind(this)} checked={this.state.enabled} />
            <label>{this.props.children}</label>
        </span>;
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
        return <span className={`GimbalParam op-${this.state.op}`}>
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
    		const opacity = Math.max(0.4, 1.0 - age / fade_duration);
    		const value = tsm.message.GimbalValue[0].value;
            const op = tsm.message.GimbalValue[1];
    		this.setState({ opacity, op, value });
    	}
    }
}
