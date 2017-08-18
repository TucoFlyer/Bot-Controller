import React, { Component } from 'react';
import SmoothieComponent from 'react-smoothie';
import windowSize from 'react-window-size';
import PropTypes from 'prop-types';
import BotConnection from './BotConnection';


class UnresponsiveChartBase extends Component {
    render() {
        return <div>
            <SmoothieComponent
                ref={ (s) => this.smoothie = s }
                width={this.props.width || window.innerWidth}
                height={this.props.height || 100}
                millisPerPixel={this.props.millisPerPixel || 15}
           
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

            {React.Children.map(this.props.children, child => {
                if (child.type === Series) {
                    return React.cloneElement(child, { chart: this })
                }
                return child;
            })}
        </div>;
    }
}

export const Chart = windowSize(UnresponsiveChartBase);

export class Series extends Component {
    render() {
        return null;
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection)
    }

    componentDidMount() {
        this.series = this.props.chart.smoothie.addTimeSeries({}, {
            strokeStyle: this.props.strokeStyle || '#3e8135',
            fillStyle: this.props.fillStyle,
            lineWidth: this.props.lineWidth || 1
        });
        this.lastTrigger = null;
        this.onFrame = this.onFrame.bind(this);
        this.context.botConnection.events.on('frame', this.onFrame);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('frame', this.onFrame);
    }

    onFrame(model) {
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
        if (trigger !== this.lastTrigger) {
            this.lastTrigger = trigger;
            this.series.append(timestamp, value);
        }
    }
}
