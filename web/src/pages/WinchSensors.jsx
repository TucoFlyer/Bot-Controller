import React, { Component } from 'react';
import { Button } from 'reactstrap';
import { Link } from 'react-router-dom';
import { Chart, Series } from '../BotChart';
import { ConfigSlider, forceToKg, distToMeters } from '../Config';
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
        const force_trigger = (model) => model.winches[id].message.WinchStatus[1].sensors.force.counter;
        const tick_trigger = (model) => model.winches[id].message.WinchStatus[1].tick_counter;
        const winch_timestamp = (model) => model.winches[id].local_timestamp;

        const params = !this.state.editable ? (
 
            <Button block color="warning" onClick={ () => this.setState({ editable: true }) }> Edit parameters </Button>
 
        ) : (<div>

            <h6>Accel rate, all winches (m/s&sup2;)</h6>
            <ConfigSlider item="params.accel_rate_m_per_sec2" min="0" max="2.0" step="1e-2" />

            <h6>Manual control velocity, all winches (m/s)</h6>
            <ConfigSlider item="params.manual_control_velocity_m_per_sec" min="0" max="2.0" step="1e-2" />

            <h6>Min force, all winches (kgf)</h6>
            <ConfigSlider item="params.force_min_kg" min="0" max="5" step="1e-2" />

            <h6>Max force, all winches (kgf)</h6>
            <ConfigSlider item="params.force_max_kg" min="0" max="5" step="1e-2" />

            <h6>Force filter param, all winches (unitless)</h6>
            <ConfigSlider item="params.force_filter_param" min="0.0" max="0.2" step="1e-4" />

        </div>);

        return <div>

            <h6>Force feedback, limits (kgf)</h6>
            <Chart>
                <Series
                    noBounds strokeStyle='#b8383d'
                    value={ (model) => forceToKg(model, id, model.winches[id].message.WinchStatus[1].command.force_min) }
                    trigger={force_trigger} timestamp={winch_timestamp} />
                <Series
                    noBounds strokeStyle='#b8383d'
                    value={ (model) => forceToKg(model, id, model.winches[id].message.WinchStatus[1].command.force_max) }
                    trigger={force_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    strokeStyle='#bbb'
                    value={ (model) => forceToKg(model, id, model.winches[id].message.WinchStatus[1].sensors.force.measure) }
                    trigger={force_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    strokeStyle='#71b1b3'
                    value={ (model) => forceToKg(model, id, model.winches[id].message.WinchStatus[1].sensors.force.filtered) }
                    trigger={force_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>Velocity feedback, target, ramp (m/s)</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].sensors.velocity) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    strokeStyle="#bbb"
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].motor.ramp_velocity) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    strokeStyle="#b28a70"
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].command.velocity_target) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <h6>Position feedback (m)</h6>
            <Chart>
                <Series
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].sensors.position) }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>

            <IfAuthenticated>{ params }</IfAuthenticated>
            <IfAuthenticated><Button block color="warning" to={`/winch/${id}/cal`} tag={Link}> Calibrate sensors </Button></IfAuthenticated>
            <Button block color="secondary" to={`/winch/${id}/timing`} tag={Link}> Timing check </Button> {" "}
        </div>;
    }
}