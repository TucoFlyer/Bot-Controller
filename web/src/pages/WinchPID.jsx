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

            <h6>Proportional gain, all winches</h6>
            <ConfigSlider item="params.pwm_gain_p" min="0" max="20.0" step="1e-3" />

            <h6>Integral gain, all winches</h6>
            <ConfigSlider item="params.pwm_gain_i" min="0" max="20.0" step="1e-3" />

            <h6>Derivative gain, all winches</h6>
            <ConfigSlider item="params.pwm_gain_d" min="0" max="20.0" step="1e-5" />

            <h6>Velocity error filter rate, all winches</h6>
            <ConfigSlider item="params.vel_err_filter_param" min="0.0" max="0.05" step="1e-6" />

            <h6>Integral error decay rate, all winches</h6>
            <ConfigSlider item="params.integral_err_decay_param" min="0.0" max="0.05" step="1e-6" />

        </div>);

        return <div>

            <h6>Position error (m)</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.position_err) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>Integral error (m&middot;s)</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.pos_err_integral) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>Velocity error, and filter (m/s)</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    strokeStyle="#bbb"
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.vel_err_inst) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    strokeStyle='#71b1b3'
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.vel_err_filtered) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>PWM command [-1,1]</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.total }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>PID contributions</h6>
            <Chart>
                <Series
                    strokeStyle='#f44'
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.p }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    strokeStyle='#4f4'
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.i }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    strokeStyle='#44f'
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.d }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <IfAuthenticated>{ params }</IfAuthenticated>
        </div>;
    }
}

