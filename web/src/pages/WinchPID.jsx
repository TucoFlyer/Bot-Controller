import React from 'react';
import { Chart, Series } from '../BotChart';
import { ConfigSlider } from '../Config';
import { IfAuthenticated } from '../BotConnection';

export default (props) => {
    const id = parseInt(props.match.params.winchId, 10);
    const tick_trigger = (model) => model.winches[id].message.WinchStatus[1].tick_counter;
    const winch_timestamp = (model) => model.winches[id].local_timestamp;

    return <div>

        <h6>Velocity error</h6>
        <Chart>
            <Series
                value={ () => 0 } strokeStyle='#aaa'
                trigger={tick_trigger} timestamp={winch_timestamp} />
            <Series
                fullDataRate
                value={ (model) => model.winches[id].message.WinchStatus[1].motor.vel_err }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>

        <h6>Velocity error derivative</h6>
        <Chart>
            <Series
                value={ () => 0 } strokeStyle='#aaa'
                trigger={tick_trigger} timestamp={winch_timestamp} />
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].motor.vel_err_diff }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>

        <h6>Velocity error integral</h6>
        <Chart>
            <Series
                value={ () => 0 } strokeStyle='#aaa'
                trigger={tick_trigger} timestamp={winch_timestamp} />
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].motor.vel_err_integral }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>

        <h6>PWM command</h6>
        <Chart>
            <Series
                value={ () => 0 } strokeStyle='#aaa'
                trigger={tick_trigger} timestamp={winch_timestamp} />
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>

        <IfAuthenticated><div>

            <h6>Proportional gain, all winches</h6>
            <ConfigSlider item="params.pwm_gain_p" min="0" max="1e-5" step="1e-9" />

            <h6>Integral gain, all winches</h6>
            <ConfigSlider item="params.pwm_gain_i" min="0" max="4e-5" step="1e-9" />

            <h6>Derivative gain, all winches</h6>
            <ConfigSlider item="params.pwm_gain_d" min="0" max="5e-7" step="1e-9" />

        </div></IfAuthenticated>
    </div>
}
