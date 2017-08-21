import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import JSONPretty from 'react-json-pretty';
import './NetworkModel.css'

export default class NetworkModel extends Component {
    constructor() {
        super();
        this.state = {
            model: {},
        };
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    render() {
        return (
            <div className="NetworkModel">
                <JSONPretty json={this.state.model} /> 
            </div>
        );
    }

    componentDidMount() {
        this.context.botConnection.events.on('frame', this.handleFrame);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('frame', this.handleFrame);
    }

    handleFrame = (model) => {
        this.setState({ model });
    }
}
