import React, { Component } from 'react';
import EventEmitter from 'events';
import nipplejs from 'nipplejs';
import './Joystick.css';

export default class Joystick extends Component {
    constructor(props) {
        super(props);
        this.events = new EventEmitter();
        if (props.onStart) this.events.on('start', props.onStart);
        if (props.onEnd) this.events.on('end', props.onEnd);
        if (props.onMove) this.events.on('move', props.onMove);
        if (props.onXY) this.events.on('xy', props.onXY);
    }

    render() {
        return (
            <div className="Joystick">
                <div className="zone" ref={ (div) => this.div = div }>
                    <div className="big">Joystick Area</div>
                    <div>touch here</div>
                </div>
            </div>
        );
    }

    componentDidMount() {
        const options = Object.assign({
            zone: this.div,
        }, this.props.options || {});
        this.manager = nipplejs.create(options);
        this.manager.on('start', this.handleStart);
        this.manager.on('end', this.handleEnd);
        this.manager.on('move', this.handleMove);
    }

    componentWillUnmount() {
        this.manager.destroy();
    }

    handleStart = (event) => {
        this.events.emit('start', event);
    }

    handleEnd = (event) => {
        this.events.emit('end', event);
        this.events.emit('xy', 0.0, 0.0);
    }

    handleMove = (event, data) => {
        this.events.emit('move', event, data);
        this.events.emit('xy',
            0.5 * data.force * Math.cos(data.angle.radian),
            0.5 * data.force * Math.sin(data.angle.radian)
        );
    }
}
