import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import Joystick from '../Joystick';
import { ConfigTextBlock } from '../Config';
import { IfAuthenticated } from '../BotConnection';

export default class FlyerHome extends Component {
    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    render() {
        return <div>     

            <h6>Flyer Mode:</h6>
            <ConfigTextBlock item="mode" />

            <IfAuthenticated><div>
                <h6>Manual tracking control</h6>
                <Joystick
                    onXY={ (x, y) => {
                        this.context.botConnection.send({ Command: { ManualControlValue: [ "CameraYaw", x ] }});
                        this.context.botConnection.send({ Command: { ManualControlValue: [ "CameraPitch", y ] }});
                    }}
                />
            </div></IfAuthenticated>

        </div>;
    }
}

