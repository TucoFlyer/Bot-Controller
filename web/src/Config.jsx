import React, { Component } from 'react';
import JSONPretty from 'react-json-pretty';
import PropTypes from 'prop-types';
import { BotConnection } from './BotConnection';

// Higher order component that adds the configuration as a prop on the wrapped component
export const withConfig = (ComposedComponent) => class extends Component {

    constructor() {
        super();
        this.state = { config: null };
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    handleConfig = (event) => {
        this.setState({ config: event.message.ConfigIsCurrent });
    }

    componentDidMount() {
        this.context.botConnection.events.on('config', this.handleConfig);
        if (this.context.botConnection.model.config) {
            this.handleConfig(this.context.botConnection.model.config);
        }
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('config', this.handleConfig);
    }

    render() {
        if (this.state.config !== null) {
            return <ComposedComponent {...this.props} config={this.state.config} />;
        } else {
            return null;
        }
    }
};

function getByPath(config, path) {
    for (let part of path.split ? path.split(".") : path) {
        config = config[part];
    }
    return config;
}

function setByPath(config, path, item) {
    let parts = Array.from(path.split ? path.split(".") : path);
    const last = parts.pop();
    for (let part of parts) {
        config = config[part];
    }
    config[last] = item;
}

// JSON text box displaying a config item
export const ConfigText = withConfig(class extends Component {
    render() {
        const value = getByPath(this.props.config, this.props.item);
        if (value instanceof Object) {
            return <JSONPretty {...this.props} json={value} />;
        } else {
            return <div {...this.props}>{ value }</div>;
        }
    }
});


