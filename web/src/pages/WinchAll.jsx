import React, { Component } from 'react';
import { Chart, Series } from '../BotChart';
import { withConfig, forceToKg, distToMeters } from '../Config';

const colors = [ '#95172f', '#951776', '#641795', '#172095', '#179195', '#17953b', '#919517', '#955017' ]


export default withConfig( class extends Component {
    force_series() {
        let result = [];
        for (let id in this.props.config.winches) {
            result.push(<Series
                strokeStyle={colors[id]}
                value={ (model) => forceToKg(model, id, model.winches[id].message.WinchStatus[1].sensors.force.filtered) }
                    trigger={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.counter }
                    timestamp={(model) => model.winches[id].local_timestamp }
            />);
        }
        return result;
    }

    velocity_series() {
        let result = [];
        for (let id in this.props.config.winches) {
            result.push(<Series
                strokeStyle={colors[id]}
                value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].sensors.velocity) }
                trigger={(model) => model.winches[id].message.WinchStatus[1].tick_counter}
                timestamp={(model) => model.winches[id].local_timestamp}
            />);
        }
        return result;
    }

    pwm_series() {
        let result = [];
        for (let id in this.props.config.winches) {
            result.push(<Series
                strokeStyle={colors[id]}
                value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm.total }
                trigger={(model) => model.winches[id].message.WinchStatus[1].tick_counter}
                timestamp={(model) => model.winches[id].local_timestamp}
            />);
        }
        return result;
    }

    render () {
        return <div>

            <h6>Force feedback, limits (kgf)</h6>
            <Chart>{ this.force_series() }</Chart>

            <h6>Velocity feedback (m/s)</h6>
            <Chart>{ this.velocity_series() }</Chart>

            <h6>Motor PWM level [-1,1]</h6>
            <Chart>{ this.pwm_series() }</Chart>

        </div>;
    }
})
