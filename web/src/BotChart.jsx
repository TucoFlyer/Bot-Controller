import React, { Component } from 'react';
import SmoothieComponent from 'react-smoothie';
import windowSize from 'react-window-size';
import PropTypes from 'prop-types';
import { BotConnection, BotModel } from './BotConnection';

export const Chart = windowSize( class extends Component {
    // Lots of opinionated defaults for Smoothie here.
    // Some of it is to make the charts fit in with our visual theme.
    // Disable bezier interpolation by default; it looks cool but hinders analysis and eats CPU
    render() {
        return <div>
            <SmoothieComponent
                ref={ (s) => this.reactSmoothie = s }
                width={this.props.width || window.innerWidth}
                height={this.props.height || 100}
                millisPerPixel={this.props.millisPerPixel || 15}
                interpolation={this.props.interpolation || 'linear'}

                grid={Object.assign({
                    fillStyle: '#fff',
                    strokeStyle: 'rgba(166,197,103,0.20)',
                    sharpLines: false,
                    millisPerLine: 1000,
                    verticalSections: 4,
                    borderVisible: true,
                }, this.props.grid || {})}
           
                labels={Object.assign({
                    fillStyle: '#444',
                }, this.props.labels || {})}
            />
            { this.props.children }
        </div>;
    }

    static childContextTypes = {
        chart: PropTypes.instanceOf(Component)
    }

    getChildContext() {
        return { chart: this };
    }
});

export class Series extends Component {
    render() {
        return null;
    }

    static propTypes ={
        noBounds: PropTypes.bool
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
        chart: PropTypes.instanceOf(Component)
    }

    componentDidMount() {
        this.series = this.context.chart.reactSmoothie.addTimeSeries({
            resetBounds: !this.props.noBounds,
            resetBoundsInterval: this.props.resetBoundsInterval || 3000,
        }, {
            strokeStyle: this.props.strokeStyle || '#3e8135',
            fillStyle: this.props.fillStyle,
            lineWidth: this.props.lineWidth || 1,
        });
        this.lastTrigger = null;

        if (this.props.fullDataRate) {
            // Store data for each message in a batch potentially (more detail)
            this.onMessages = this.onMessages.bind(this);
            this.context.botConnection.events.on('messages', this.onMessages);
        } else {
            // Only evaluate the model once per frame (smoother)
            this.onFrame = this.onFrame.bind(this);
            this.context.botConnection.events.on('frame', this.onFrame);
        }
    }

    componentWillUnmount() {
        if (this.context.chart.reactSmoothie) {
            this.context.chart.reactSmoothie.smoothie.removeTimeSeries(this.series);
        }
        this.context.botConnection.events.removeListener('messages', this.onMessages);
        this.context.botConnection.events.removeListener('frame', this.onFrame);
    }

    onFrame(model) {
        this.updateFromModel(model);
    }

    onMessages(messages) {
        let model = new BotModel();
        for (let msg of messages) {
            model.update(msg);
            this.updateFromModel(model);
        }
    }

    updateFromModel(model) {
        let value, timestamp, trigger;
        try {
            // Most recent value
            value = this.props.value(model);
            // When the packet arrived (relevant data may or may not be new)
            timestamp = this.props.timestamp(model);
            // Trigger for updates (indicates that data has been refreshed)
            trigger = this.props.trigger(model);
        }
        catch (e) {
            return;
        }
        if (trigger !== null && trigger !== undefined && trigger !== this.lastTrigger) {
            this.lastTrigger = trigger;

            // Directly push new data instead of using series.append,
            // we don't need or want smoothie's duplicate/reverse timestamp detection
            this.series.data.push([timestamp, value]);

            if (this.props.noBounds) {
                // Smoothie automatically sets these according to the single
                // data point even when the resetBounds timer is disabled. Undo that,
                // our noBounds option means this series shouldn't contribute
                // to the bounds at all.
                this.series.minValue = Number.NaN;
                this.series.maxValue = Number.NaN;
            } else {
                // Calculate bounds right away
                this.series.minValue = this.series.minValue < value ? this.series.minValue : value;
                this.series.maxValue = this.series.maxValue > value ? this.series.maxValue : value;
            }
        }
    }
}
