import React, { Component } from 'react';
import { Button } from 'reactstrap';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import { Chart, Series } from '../BotChart';

export default (props) => {
    const gimbal_status_timestamp = (model) => model.gimbal_status.local_timestamp;

    return <div>

        <h4>Motor Control</h4>

        <GimbalMotorButton block color="warning" enable={true}>
            Stabilization motors ON
        </GimbalMotorButton>

        <GimbalMotorButton block color="secondary" enable={false}>
            Stabilization motors OFF
        </GimbalMotorButton>

        <h4>Gimbal controller status</h4>

        <h6>Yaw angle</h6>
        <Chart>
            <Series
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.angles[0] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
        </Chart>

        <h6>Pitch angle</h6>
        <Chart>
            <Series
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.angles[1] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
        </Chart>

        <h6>Yaw/Pitch rate control</h6>
        <Chart>
            <Series
                strokeStyle='#a22'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.rates[0] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
            <Series
                strokeStyle='#22a'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.rates[1] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
        </Chart>

        <h6>Tracker output, proportional (P) control rate</h6>
        <Chart>
            <Series
                strokeStyle='#a22'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.tracking_p_rates[0] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
            <Series
                strokeStyle='#22a'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.tracking_p_rates[1] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
        </Chart>

        <h6>Tracker output, integrated (I) control rate</h6>
        <Chart>
            <Series
                strokeStyle='#a22'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.tracking_i_rates[0] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
            <Series
                strokeStyle='#22a'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.tracking_i_rates[1] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
        </Chart>

    </div>;
}

class GimbalMotorButton extends Component {
    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    render() {
        const { enable, children, ...props } = this.props;
        return <Button {...props} onClick={() => {
            this.context.botConnection.socket.send(JSON.stringify({
                Command: { GimbalMotorEnable: enable }
            }));
            }}> { children }
        </Button>;
    }
}
