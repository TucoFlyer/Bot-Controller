import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import Joystick from '../Joystick';
import { ConfigTextBlock, ConfigButton } from '../Config';
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
                <ConfigButton item="mode" value="Halted" block color="danger" > Halt </ConfigButton>
            </div></IfAuthenticated>

            <IfAuthenticated><div>
                <h6>Manual camera control</h6>
                <Joystick
                    onXY={ (x, y) => {
                        this.context.botConnection.send({ Command: { ManualControlValue: [ "CameraYaw", x ] }});
                        this.context.botConnection.send({ Command: { ManualControlValue: [ "CameraPitch", y ] }});
                    }}
                />
                <ConfigButton item="mode" value="Halted" block color="danger" > Halt </ConfigButton>
            </div></IfAuthenticated>

            <IfAuthenticated><div>
                <h6>Manual flyer control, XY plane (all winches)</h6>
                <Joystick
                    onStart={ () => {
                        this.context.botConnection.send({ Command: { SetMode: "ManualFlyer" }});
                    }}
                    onXY={ (x, y) => {
                        this.context.botConnection.send({ Command: { ManualControlValue: [ "RelativeX", x ] }});
                        this.context.botConnection.send({ Command: { ManualControlValue: [ "RelativeY", y ] }});
                    }}
                />
                <ConfigButton item="mode" value="Halted" block color="danger" > Halt </ConfigButton>
            </div></IfAuthenticated>

            <IfAuthenticated><div>
                <h6>Manual flyer control, Z axis (all winches)</h6>
                <Joystick
                    onStart={ () => {
                        this.context.botConnection.send({ Command: { SetMode: "ManualFlyer" }});
                    }}
                    onXY={ (x, y) => {
                        this.context.botConnection.send({ Command: { ManualControlValue: [ "RelativeZ", y ] }});
                    }}
                />
                <ConfigButton item="mode" value="Halted" block color="danger" > Halt </ConfigButton>
            </div></IfAuthenticated>

        </div>;
    }
}

