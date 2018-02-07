import React, { Component } from 'react';
import { Chart, Series } from '../BotChart';
import Gauge from '../BotGauge';
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
        return <div>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Manual control velocity, all winches (m/s)</h6>
                <ConfigSlider item="params.manual_control_velocity_m_per_sec" min="0" max="2.0" step="1e-2" />
            </div> }</IfAuthenticated>
            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Acceleration limit, all winches (m/s&sup2;)</h6>
                <ConfigSlider item="params.accel_limit_m_per_sec2" min="0" max="1.0" step="1e-2" />
            </div> }</IfAuthenticated>

            <h6>Position error, and filter (m)</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    strokeStyle="#bbb"
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.position_err) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    strokeStyle="#71b1b3"
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.pos_err_filtered) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Deadband position error, all winches (m)</h6>
                <ConfigSlider item="params.deadband_position_err_m" min="0.0" max="0.04" step="1e-4" />
            </div> }</IfAuthenticated>
            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Deadband velocity, all winches (m)</h6>
                <ConfigSlider item="params.deadband_velocity_limit_m_per_sec" min="0.0" max="0.04" step="1e-4" />
            </div> }</IfAuthenticated>
            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Position error filter rate, all winches</h6>
                <ConfigSlider item="params.pos_err_filter_param" min="0.0" max="0.4" step="1e-6" />
            </div> }</IfAuthenticated>


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

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Integral error decay rate, all winches</h6>
                <ConfigSlider item="params.integral_err_decay_param" min="0.0" max="0.01" step="1e-6" />
            </div> }</IfAuthenticated>

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

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Velocity error filter rate, all winches</h6>
                <ConfigSlider item="params.vel_err_filter_param" min="0.0" max="0.05" step="1e-6" />
            </div> }</IfAuthenticated>

            <h6>PWM command [-1,1]</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.total }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>
            <Gauge
                value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.total}
                minValue={-1.0} maxValue={1.0}
                majorTicks={[-1, -0.5, 0, 0.5, 1]}
                highlights={[
                    { from: -1, to: -0.9, color: "rgba(190,0,0,0.3)" },
                    { from: -0.9, to: -0.1, color: "rgba(255,255,0,0.3)" },
                    { from: -0.1, to: 0.1, color: "rgba(127,127,127,0.3)" },
                    { from: 0.1, to: 0.9, color: "rgba(255,255,0,0.3)" },
                    { from: 0.9, to: 1.0, color: "rgba(190,0,0,0.3)" },
                ]}
            />

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Proportional gain, all winches</h6>
                <ConfigSlider item="params.pwm_gain_p" min="0" max="10.0" step="1e-3" />
            </div> }</IfAuthenticated>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Integral gain, all winches</h6>
                <ConfigSlider item="params.pwm_gain_i" min="0" max="10.0" step="1e-3" />
            </div> }</IfAuthenticated>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Derivative gain, all winches</h6>
                <ConfigSlider item="params.pwm_gain_d" min="0" max="1.0" step="1e-5" />
            </div> }</IfAuthenticated>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>PWM output minimum, all winches</h6>
                <ConfigSlider item="params.pwm_minimum" min="0" max="1.0" step="1e-5" />
            </div> }</IfAuthenticated>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>PWM output bias, all winches</h6>
                <ConfigSlider item="params.pwm_bias" min="0" max="1.0" step="1e-5" />
            </div> }</IfAuthenticated>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>PWM frequency during low-speed motion, all winches (Hz)</h6>
                <ConfigSlider item="params.pwm_hz_low_motion" min="100" max="1500" step="0.1" />
            </div> }</IfAuthenticated>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>PWM frequency during high-speed motion, all winches (Hz)</h6>
                <ConfigSlider item="params.pwm_hz_high_motion" min="1500" max="40000" step="0.1" />
            </div> }</IfAuthenticated>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>PWM frequency filter parameter, all winches</h6>
                <ConfigSlider item="params.pwm_hz_filter_param" min="0" max="0.1" step="1e-5" />
            </div> }</IfAuthenticated>

            <IfAuthenticated>{ this.state.editable && <div>
                <h6>Velocity threshold for PWM frequency shift, all winches (m/s)</h6>
                <ConfigSlider item="params.pwm_velocity_threshold" min="0" max="0.1" step="1e-5" />
            </div> }</IfAuthenticated>

            <h6>PID contributions</h6>
            <Chart>
                <Series
                    strokeStyle='#a22'
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.p }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    strokeStyle='#2a2'
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.i }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    strokeStyle='#22a'
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.d }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>PWM Frequency (Hz)</h6>
            <Chart>
                <Series
                    value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.hz }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <IfAuthenticated>{ !this.state.editable && <div>
                <Button block color="warning" onClick={ () => this.setState({ editable: true }) }>
                    Edit parameters
                </Button>
            </div> }</IfAuthenticated>

        </div>;
    }
}

