import React, { Component } from 'react';
import PropTypes from 'prop-types';
import EventEmitter from 'events';
import hmacSHA512 from 'crypto-js/hmac-sha512';
import Base64 from 'crypto-js/enc-base64';
import fetch from 'isomorphic-fetch';
import ReconnectingWebSocket from 'reconnecting-websocket';


export default class BotConnection extends Component {

    static childContextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection)
    }

    constructor() {
        super();
        this.events = new EventEmitter();
        this.socket = null;
        this.frame_request = null;
    }

    getChildContext() {
        return { botConnection: this };
    }

    getWebsocketInfo() {
        // The real server has an HTTP API for getting the WebSocket URI.
        // But if that fails, make a guess that will work for development with "npm start" or whatever.

        return fetch('/ws').then((response) => {
            return response.json();
        }).catch((err) => {
            console.log(`Guessing WebSocket config, failed to use HTTP API (${err})`);
            return { uri: `ws://${window.location.hostname}:8081` };
        });
    }

    handleSocketMessage(evt) {
        var json = JSON.parse(evt.data);
        var time_offset = null;
        var last_timestamp = null;
        var model = {
            flyer: null,
            winches: [],
        };

        if (Array.isArray(json) && json.length > 0) {

            // Update time offset from first message, restart if timestamps go backward.
            if (json[0].timestamp < last_timestamp) {
                time_offset = null;
            }
            last_timestamp = json[0].timestamp;
            if (time_offset === null) {
                time_offset = new Date().getTime() - json[0].timestamp;
            }

            // Annotate all messages with local timestamp, and update the model
            for (var msg of json) {
                msg.local_timestamp = time_offset + msg.timestamp;

                if (msg.message.WinchStatus) {
                    model.winches[msg.message.WinchStatus[0]] = msg;
                }

                if (msg.message.FlyerSensors) {
                    model.flyer = msg;
                }
            }

            // Event for access to a raw message burst
            this.events.emit('messages', json);

            // Batch messages into UI frames
            if (!this.frame_request) {
                this.frame_request = window.requestAnimationFrame(() => {
                    this.frame_request = null;
                    this.events.emit('frame', model);
                });
            }

        } else if (json.challenge) {
            // Authentication challenge
            this.handleChallenge(json);

        } else {
            console.log("Unrecognized message ", json);
        }
    }

    handleChallenge(msg) {
        var key = window.location.hash.substring(1);
        if (key.length >= 1) {
            var digest = Base64.stringify(hmacSHA512(msg.challenge, key))
            this.socket.send(JSON.stringify({ authenticate: digest }));
        }
    }

    componentDidMount() {
        this.getWebsocketInfo().then((ws) => {
            this.socket = new ReconnectingWebSocket(ws.uri, undefined, {connectionTimeout: 1000});
            this.socket.addEventListener('message', this.handleSocketMessage.bind(this));
        });
    }

    componentWillUnmount() {
        if (this.socket) {
            this.socket.close();
            this.socket = null;
        }
        if (this.frame_request) {
            window.cancelAnimationFrame(this.frame_request);
            this.frame_request = null;
        }
    }

    render() {
        return <div> {this.props.children} </div>;
    }
}
