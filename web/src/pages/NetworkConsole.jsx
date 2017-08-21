import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import './NetworkConsole.css';

export default class NetworkConsole extends Component {
    constructor() {
        super();
        this.msgId = 0;
        this.state = {
            entry: "",
            log: [],
        };
    }

    static maxLogLines = 100;

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    render() {
        return (
            <div className="NetworkConsole">
                <form onSubmit={this.handleSubmit}>
                    <input type="text" value={this.state.entry} onChange={this.handleChangeEntry} />
                </form>
                <div>
                    { this.state.log }
                </div>
            </div>
        );
    }

    componentDidMount() {
        this.context.botConnection.events.on('log', this.handleLog);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('log', this.handleLog);
    }

    handleLog = (msg) => {
        this.msgId += 1;
        const log = [
            <div key={`msg-${this.msgId}`}>
                { JSON.stringify(msg) }
            </div>
        ].concat(this.state.log.slice(this.maxLogLines));
        this.setState({ log });
    }

    handleChangeEntry = (event) => {
        this.setState({ entry: event.target.value });
    }

    handleSubmit = (event) => {
        this.context.botConnection.socket.send(this.state.entry);
        this.setState({ entry: "" });
        event.preventDefault();
    }
}
