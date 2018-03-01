import React, { Component } from 'react';
import Gauges from 'canvas-gauges';
import PropTypes from 'prop-types';
import { BotConnection } from './BotConnection';
import JSONPretty from 'react-json-pretty';

export default class BotJSON extends Component {

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    constructor() {
        super();
        this.state = { value: undefined };
    }

    render() {
        return <JSONPretty json={this.state.value} />;
    }

    componentDidMount() {
        this.context.botConnection.events.on('frame', this.handleFrame);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('frame', this.handleFrame);
    }

    handleFrame = (model) => {
        var value;
        try {
            value = this.props.value(model);
        }
        catch (e) {
            return;
        }
        this.setState({ value });
    }
}
