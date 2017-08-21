import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import JSONPretty from 'react-json-pretty';
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

    addToLog = (element) => {
        this.msgId += 1;
        const log = [
            <div key={`msg-${this.msgId}`}>
                { element }
            </div>
        ].concat(this.state.log.slice(this.maxLogLines));
        this.setState({ log });
    }

    handleLog = (msg, className) => {
        this.addToLog(<JSONPretty json={msg} />);
    }

    handleChangeEntry = (event) => {
        this.setState({ entry: event.target.value });
    }

    handleSubmit = (event) => {
        const msg = this.state.entry;
        setTimeout(() => { document.execCommand('selectall', null, false); });
        event.preventDefault();

        this.context.botConnection.socket.send(msg);
        this.addToLog(<pre class="sent"> {msg} </pre>);
    }
}
