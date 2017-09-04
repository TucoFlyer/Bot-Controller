import React, { Component } from 'react';
import { ConfigSlider, ConfigTextBlock, ConfigButton, withConfig } from '../Config';
import { ButtonToolbar } from 'reactstrap';
import './LightingSchemes.css';

export default class extends Component {
    render() {
        return <div className="LightingSchemes">

            <h4>Brightness</h4>

            <ConfigSlider item="lighting.current.brightness" min="0" max="2.0" step="1e-2" />
            <ButtonToolbar>
                <ConfigButton item="lighting.current.brightness" value="0.0" color="secondary">Off</ConfigButton>
                <ConfigButton item="lighting.current.brightness" value="0.2" color="muted">Dim</ConfigButton>
                <ConfigButton item="lighting.current.brightness" value="1.0" color="primary">Normal</ConfigButton>
                <ConfigButton item="lighting.current.brightness" value="1.5" color="info">Bright</ConfigButton>
            </ButtonToolbar>

            <h4>Saved Lighting Schemes</h4>
            <SavedSchemes />
            <NewScheme />

            <h4>Lighting schedule</h4>
            <ConfigTextBlock item="lighting.schedule" />

        </div>;
    }
}

const SavedSchemes = withConfig(class extends Component {
    render() {
        let schemes = [];
        const current_scheme = this.props.config.lighting.current;
        for (let name in this.props.config.lighting.saved) {
            const this_scheme = this.props.config.lighting.saved[name];
            const is_current = JSON.stringify(this_scheme) === JSON.stringify(current_scheme);
            schemes.push(
                <ButtonToolbar>
                    <ConfigButton
                        color="warning"
                        item={["lighting", "saved", name]}
                        value={null}>
                        Delete
                    </ConfigButton>
                    <ConfigButton
                        color="warning"
                        item={["lighting", "saved", name]}
                        value={current_scheme}>
                        Update
                    </ConfigButton>
                    <ConfigButton
                        color="primary"
                        item="lighting.current"
                        value={this_scheme}>
                        Apply "{name}"
                    </ConfigButton>
                    { is_current && <span className="current">
                        &#9664; Current
                    </span> }
                </ButtonToolbar>
            );
        }
        return <div className="SavedSchemes">{schemes}</div>;
    }
});

const NewScheme = withConfig(class extends Component {
    constructor() {
        super();
        this.state ={
            name: "Untitled"
        }
    }

    render() {
        return <div className="NewScheme">
            <ButtonToolbar>
                <ConfigButton
                    item={["lighting", "saved", this.state.name]}
                    value={this.props.config.lighting.current}>
                    Save As:
                </ConfigButton>
                <input
                    type="text"
                    value={this.state.name}
                    onChange={this.handleChange.bind(this)}>
                    </input>
            </ButtonToolbar>
        </div>;
    }

    handleChange(event) {
        this.setState({ name: event.target.value });
    }
});
