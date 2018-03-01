import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import { IfAuthenticated } from '../BotConnection';
import BotJSON from '../BotJSON';
import { Button } from 'reactstrap';

export default class extends Component {
    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    render() {
        return <div>

            <h3>Local recording</h3>

            <BotJSON value={ (model) => model.camera.outputs.message.Command.CameraOutputStatus.LocalRecording } />

            <IfAuthenticated><div><Button block color="warning" onClick={ () => {
                this.context.botConnection.socket.send(JSON.stringify({
                    Command: {CameraOutputEnable: ["LocalRecording", false]}
                }));
            }}> Stop Recording </Button></div></IfAuthenticated>

            <IfAuthenticated><div><Button block color="danger" onClick={ () => {
                this.context.botConnection.socket.send(JSON.stringify({
                    Command: {CameraOutputEnable: ["LocalRecording", true]}
                }));
            }}> Start Recording </Button></div></IfAuthenticated>

            <h3>Live stream</h3>

            <BotJSON value={ (model) => model.camera.outputs.message.Command.CameraOutputStatus.LiveStream } />

            <IfAuthenticated><div><Button block color="warning" onClick={ () => {
                this.context.botConnection.socket.send(JSON.stringify({
                    Command: {CameraOutputEnable: ["LiveStream", false]}
                }));
            }}> Stop Streaming </Button></div></IfAuthenticated>

            <IfAuthenticated><div><Button block color="danger" onClick={ () => {
                this.context.botConnection.socket.send(JSON.stringify({
                    Command: {CameraOutputEnable: ["LiveStream", true]}
                }));
            }}> Start Streaming </Button></div></IfAuthenticated>

        </div>;
    }
}

