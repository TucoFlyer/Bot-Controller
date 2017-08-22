import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from './BotConnection';
import { Badge } from 'reactstrap';

export default class AuthStatus extends Component {
    render() {
        const state = this.context.botConnection.state;
        if (state.authenticated) {
            return <Badge {...this.props} color="success">Authenticated</Badge>;
        }
        if (!state.connected) {
            return <Badge {...this.props} color="danger">Disconnected</Badge>;
        }
        return <Badge {...this.props} color="secondary">Guest</Badge>;
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }
}
