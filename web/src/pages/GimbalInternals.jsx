import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import reactCSS from 'reactcss';

export default (props) => {
	let params = [];
	for (let i = 0; i < 128; i++) {
		let row = [];
		for (let t = 0; t < 3; t++) {
			row.push(<GimbalParam index={i} target={t} key={`gimbal-param-${i}-${t}`} />);
		}
		params.push(<div className="Gimbal" key={`gimbal-param-${i}`} >
			<span className="index">{ ("00" + i.toString(16)).slice(-2) }</span>
			{row}
		</div>);
	}

    return <div>

        <h4>Firmware parameters</h4>
        <div> { params } </div>

    </div>;
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
