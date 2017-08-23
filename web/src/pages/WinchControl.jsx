import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import Joystick from '../Joystick';

export default class WinchControl extends Component {
    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    render() {
        const id = parseInt(this.props.match.params.winchId, 10);
        return <div>
            <h6>Velocity control, direct to Bot {id}</h6>
            <Joystick
                onStart={ () => {
                    this.context.botConnection.send({ Command: { SetMode: { ManualWinch: id }}});
                }}
                onXY={ (x, y) => {
                    this.context.botConnection.send({ Command: { ManualControlValue: [ "RelativeX", x ] }});
                    this.context.botConnection.send({ Command: { ManualControlValue: [ "RelativeY", y ] }});
                }}
            />
        </div>;
    }
}
