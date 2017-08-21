import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from './BotConnection';
import { Badge } from 'reactstrap';
import './AuthStatus.css';

export default class AuthStatus extends Component {
    render() {
        const state = this.context.botConnection.state;
        const badge =
            state.authenticated ? <Badge color="success">Authenticated</Badge> :
            !state.connected ? <Badge color="danger">Disconnected</Badge> :
            <Badge color="info">Guest</Badge>;
        return <div className="AuthStatus">{badge}</div>;
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }
}
