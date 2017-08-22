import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection, IfAuthenticated } from '../BotConnection';
import Joystick from '../Joystick';

export default class WinchControl extends Component {
    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    render() {
        const id = parseInt(this.props.match.params.winchId, 10);
        return (
            <IfAuthenticated>
                <div>
                    <h6>Velocity control, direct to Bot {id}</h6>
                    <Joystick
                        onStart={ (event, data) => {
                            this.setManualMode(id);
                        }}
                        onXY={ (x, y) => {
                            this.control(y);
                        }}
                    />
                </div>
           </IfAuthenticated>
        );
    }

    setManualMode(winchId) {
        const bot = this.context.botConnection;
        bot.send({ Command: { SetMode: { ManualWinch: winchId }}});
    }

    control(value) {
        const bot = this.context.botConnection;
        bot.send({ Command: { ManualControlValue: [ "RelativeY", value ] }});
    }
}
