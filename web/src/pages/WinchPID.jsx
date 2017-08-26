import React, { Component } from 'react';
import { Chart, Series } from '../BotChart';
import { ConfigSlider, distToMeters } from '../Config';
import { Button } from 'reactstrap';
import { IfAuthenticated } from '../BotConnection';

export default class extends Component {
    constructor() {
        super();
        this.state = {
            editable: false,
        };
    }

    render () {
        const id = parseInt(this.props.match.params.winchId, 10);
        const tick_trigger = (model) => model.winches[id].message.WinchStatus[1].tick_counter;
        const winch_timestamp = (model) => model.winches[id].local_timestamp;

        const params = !this.state.editable ? (

            <Button block color="warning" onClick={ () => this.setState({ editable: true }) }> Edit gains </Button>

        ) : (<div>

            <h6>Velocity derivative filter rate, all winches</h6>
            <ConfigSlider item="params.diff_filter_param" min="0.0" max="0.2" step="1e-6" />

            <h6>Proportional gain, all winches</h6>
            <ConfigSlider item="params.pwm_gain_p" min="0" max="0.4" step="1e-3" />

            <h6>Integral gain, all winches</h6>
            <ConfigSlider item="params.pwm_gain_i" min="0" max="0.4" step="1e-3" />

            <h6>Derivative gain, all winches</h6>
            <ConfigSlider item="params.pwm_gain_d" min="0" max="0.4" step="1e-5" />

        </div>);

        return <div>

            <h6>Velocity error (m/s)</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.vel_err) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>Velocity error derivative (m/s&sup2;)</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.vel_err_diff) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>Velocity error integral (m)</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.vel_err_integral) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>PWM command [-1,1]</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <IfAuthenticated>{ params }</IfAuthenticated>
        </div>;
    }
}

