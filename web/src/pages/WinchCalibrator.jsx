import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { Button } from 'reactstrap';
import JSONPretty from 'react-json-pretty';
import { ConfigSlider, ConfigRevertButton } from '../Config';
import { Chart, Series } from '../BotChart';
import { BotConnection } from '../BotConnection';

class CalibratorBase extends Component {
    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    constructor() {
        super();
        this.state = {
            next_measure: 0,
            saved: false,
            measures: [],
        };
    }

    measurementButton(index, children) {
        return <Button
            color={index === this.state.next_measure ? "primary" : "secondary"}
            disabled={index > this.state.next_measure}
            onClick={() => {
                const next_measure = index + 1;
                let measures = this.state.measures.slice(0, index);
                measures.push(this.currentMeasurement());
                this.setState({ next_measure, measures, saved: false });
            }}
        > { children } </Button>;
    }

    measurementValue(index) {
        if (index < this.state.next_measure) {
            const saved = this.state.measures[index];
            return `(${saved})`;
        }
        return null;
    }

    previewCalibration(num_measures_needed) {
        if (this.state.next_measure < num_measures_needed) {
            return null;
        }
        const cal = this.makeCalibration();
        return <div><JSONPretty json={cal} /></div>;
    }

    saveButton(num_measures_needed, children) {
        const cal = this.state.next_measure < num_measures_needed ? null : this.makeCalibration();
        const disabled = !cal || !cal.config;
        return <Button
            color={disabled || this.state.saved ? "secondary" : "warning"}
            disabled={disabled}
            onClick={() => {
                this.context.botConnection.socket.send(JSON.stringify({ UpdateConfig: cal.config }));
                this.setState({ saved: true });
            }}
        > { children } </Button>;
    }
}

class DistanceCalibrator extends CalibratorBase {
    currentMeasurement() {
        const id = parseInt(this.props.winchId, 10);
        const model = this.context.botConnection.model;
        return model.winches[id].message.WinchStatus[1].sensors.position;
    }

    makeCalibration() {
        const measures = this.state.measures;
        const delta_1 = Math.abs(measures[1] - measures[0]);
        const delta_2 = Math.abs(measures[2] - measures[1]);
        const delta_3 = Math.abs(measures[3] - measures[2]);
        const avg_delta = (delta_1 + delta_2 + delta_3) / 3;

        if (avg_delta < 1.0) {
            return { error: "No motion recorded" };
        }

        const id = parseInt(this.props.winchId, 10);
        let winches = []
        winches[id] = {
            calibration: {
                m_dist_per_count: 1.0 / avg_delta,
            }
        };
        return { config: { winches } };
    }

    render() {
        const id = parseInt(this.props.winchId, 10);
        return <ol>
            <li>Open a <a target="_blank" href={`/#/winch/${id}/control`}>winch controller</a></li>
            <li>{this.measurementButton(0, "Record")} a reference position {this.measurementValue(0)}</li>
            <li>Move the winch 1 meter in either direction</li>
            <li>{this.measurementButton(1, "Record")} a data point {this.measurementValue(1)}</li>
            <li>Move the winch another meter in either direction</li>
            <li>{this.measurementButton(2, "Record")} another data point {this.measurementValue(2)}</li>
            <li>Move the winch another meter in either direction</li>
            <li>{this.measurementButton(3, "Record")} another data point {this.measurementValue(3)}</li>
            <li>Now to calculate the new calibration. {this.previewCalibration(4)}</li>
            <li>{this.saveButton(4, "Save")} it to the controller</li>
        </ol>;
    }
}

class ForceCalibrator extends CalibratorBase {
    currentMeasurement() {
        const id = parseInt(this.props.winchId, 10);
        const model = this.context.botConnection.model;
        return model.winches[id].message.WinchStatus[1].sensors.force.filtered;
    }

    makeCalibration() {
        const measures = this.state.measures;
        const added_weight = parseFloat(this.state.added_weight);
        if (!(added_weight > 0)) {
            return { error: "Invalid added weight value" };
        }

        const avg_zero = ( measures[6] + measures[0] ) / 2;
        const delta_1 = measures[2] - measures[1];
        const delta_2 = measures[2] - measures[3];
        const delta_3 = measures[4] - measures[3];
        const delta_4 = measures[4] - measures[5];
        const avg_delta = (delta_1 + delta_2 + delta_3 + delta_4) / 4;

        const id = parseInt(this.props.winchId, 10);
        let winches = []
        winches[id] = {
            calibration: {
                force_zero_count: avg_zero,
                kg_force_per_count: added_weight / avg_delta,
            }
        };
        return { config: { winches } };
    }

    constructor() {
        super();
        this.state.added_weight = "";
    }

    handleChangeWeight = (event) => {
        this.setState({ added_weight: event.target.value });
    }

    render() {
        return <ol>
            <li>Temporarily adjust the filtering to be quite slow (near 0.0) damping any visible vibration.
                <ConfigSlider item="params.force_filter_param" min="0.000001" max="0.2" step="1e-4" /></li>
            <li>The filter should take several seconds to settle after each change. Make sure to wait for it.</li>
            <li>Pull the cord down toward the winch, letting it go slack.</li>
            <li>{this.measurementButton(0, "Record")} a zero reading {this.measurementValue(0)}</li>
            <li>Attach an arbitrary counterweight to the winch.</li>
            <li>{this.measurementButton(1, "Record")} the reference weight {this.measurementValue(1)}</li>
            <li>Add a known additional weight,&nbsp;
                <input type="text" size="6" value={this.state.added_weight} onChange={this.handleChangeWeight}>
                </input> kg</li>
            <li>{this.measurementButton(2, "Record")} the total weight {this.measurementValue(2)}</li>
            <li>Take the additional weight back off, leaving the arbitrary load.</li>
            <li>{this.measurementButton(3, "Record")} the reference again {this.measurementValue(3)}</li>
            <li>Put the measured weight back on.</li>
            <li>{this.measurementButton(4, "Record")} the total again {this.measurementValue(4)}</li>
            <li>Take it off, leaving the arbitrary load again.</li>
            <li>{this.measurementButton(5, "Record")} the reference again {this.measurementValue(5)}</li>
            <li>Finally pull the cord slack again.</li>
            <li>{this.measurementButton(6, "Record")} a final zero reading {this.measurementValue(6)}</li>
            <li>Now to calculate the new calibration. {this.previewCalibration(7)}</li>
            <li>{this.saveButton(7, "Save")} it to the controller</li>
            <li>You clobbered the force_filter_param value on all winches above, so remember
                to revert or re-adjust this setting afterward.
                <div><ConfigRevertButton
                    color={this.state.saved ? "primary" : "secondary"}
                    item="params.force_filter_param" /></div></li>

        </ol>;
    }
}

export default class WinchCalibrator extends Component {
    render() {
        const id = parseInt(this.props.match.params.winchId, 10);
        const force_trigger = (model) => model.winches[id].message.WinchStatus[1].sensors.force.counter;
        const tick_trigger = (model) => model.winches[id].message.WinchStatus[1].tick_counter;
        const winch_timestamp = (model) => model.winches[id].local_timestamp;

        return <div>

            <h4>Calibrate distance sensor</h4>
            <Chart><Series
                    value={ (model) => model.winches[id].message.WinchStatus[1].sensors.position }
                    trigger={tick_trigger} timestamp={winch_timestamp} />
            </Chart>
            <DistanceCalibrator winchId={this.props.match.params.winchId} />

            <h4>Calibrate force sensor</h4>
            <Chart><Series
                    value={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.filtered }
                    trigger={force_trigger} timestamp={winch_timestamp} />
            </Chart>
            <ForceCalibrator winchId={this.props.match.params.winchId} />

        </div>;
    }
}
