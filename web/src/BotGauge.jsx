import React, { Component } from 'react';
import Gauges from 'canvas-gauges';
import PropTypes from 'prop-types';
import { BotConnection } from './BotConnection';

export default class Gauge extends Component {

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    render() {
        return (
            <div className="Gauge">
                <canvas
                    className="Gauge"
                    ref={(c) => { this.canvas = c }}
                />
            </div>
        );
    }

    componentDidMount() {
        const cls = Gauges[this.props.type || "LinearGauge"];
        this.gauge = new cls({

            borders: false,
            borderOuterWidth: 0,
            borderMiddleWidth: 0,
            borderInnerWidth: 0,
            borderShadowWidth: 0,
            colorBorderShadow: false,
            
            barBeginCircle: false,
            
            tickSide: "right",
            needleSide: "both",
            numberSide: "right",
            
            height: 80,
            barLength: 90,
            
            colorBar: "#eee",
            colorBarProgress: false,
            
            ...this.props,
            
            value: 0,
            renderTo: this.canvas
        });
        this.context.botConnection.events.on('frame', this.handleFrame);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('frame', this.handleFrame);
    }

    handleFrame = (model) => {
        const value = this.props.value(model);
        this.gauge.update({ value });
    }
}
