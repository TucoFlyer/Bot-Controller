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

            <h6>Lockout below force, all winches (kgf)</h6>
            <ConfigSlider item="params.force_lockout_below_kg" min="0" max="5" step="1e-2" />

            <h6>Negative motion minimum force, all winches (kgf)</h6>
            <ConfigSlider item="params.force_neg_motion_min_kg" min="0" max="5" step="1e-2" />

            <h6>Positive motion maximum force, all winches (kgf)</h6>
            <ConfigSlider item="params.force_pos_motion_max_kg" min="0" max="5" step="1e-2" />

            <h6>Lockout above force, all winches (kgf)</h6>
            <ConfigSlider item="params.force_lockout_above_kg" min="0" max="5" step="1e-2" />

            <h6>Auto force return velocity maximum, all winches (m/s)</h6>
            <ConfigSlider item="params.force_return_velocity_max_m_per_sec" min="0" max="2.0" step="1e-2" />

            <h6>Force filter param, all winches (unitless)</h6>
            <ConfigSlider item="params.force_filter_param" min="0.0" max="0.2" step="1e-4" />

        </div>);

        return <div>

            <h6>Force feedback, limits (kgf)</h6>
            <Chart>
                <Series
                    noBounds strokeStyle='#b8383d'
                    value={ (model) => forceToKg(model, id, model.winches[id].message.WinchStatus[1].command.force.neg_motion_min) }
                    trigger={force_trigger} timestamp={winch_timestamp} />
                <Series
                    noBounds strokeStyle='#b8383d'
                    value={ (model) => forceToKg(model, id, model.winches[id].message.WinchStatus[1].command.force.pos_motion_max) }
                    trigger={force_trigger} timestamp={winch_timestamp} />
                <Series
                    noBounds strokeStyle='#e22'
                    value={ (model) => forceToKg(model, id, model.winches[id].message.WinchStatus[1].command.force.lockout_below) }
                    trigger={force_trigger} timestamp={winch_timestamp} />
                <Series
                    noBounds strokeStyle='#e22'
                    value={ (model) => forceToKg(model, id, model.winches[id].message.WinchStatus[1].command.force.lockout_above) }
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

            <h6>Velocity feedback (m/s)</h6>
            <Chart>
                <Series
                    value={ () => 0 } strokeStyle='#aaa'
                    trigger={tick_trigger} timestamp={winch_timestamp} />
                <Series
                    fullDataRate
                    value={ (model) => distToMeters(model, id, model.winches[id].message.WinchStatus[1].sensors.velocity) }
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