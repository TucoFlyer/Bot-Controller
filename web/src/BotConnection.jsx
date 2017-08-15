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
        var time = new Date().getTime();

        if (Array.isArray(json)) {
            // Burst of message bus activity
            for (var msg of json) {
                msg.timestamp = time;
                this.events.emit('message', msg);
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
        var digest = Base64.stringify(hmacSHA512(msg.challenge, key))
        this.socket.send(JSON.stringify({ authenticate: digest }));
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
    }

    render() {
        return <div> {this.props.children} </div>;
    }
}
