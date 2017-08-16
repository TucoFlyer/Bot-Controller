import React from 'react';
import SmoothieComponent from 'react-smoothie';
import windowSize from 'react-window-size';
import PropTypes from 'prop-types';
import BotConnection from './BotConnection';
import jp from 'jsonpath';
import './BotChart.css';

class BotChart extends React.Component {

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection)
    }

    render() {
        return (
            <div className="Chart">
                <SmoothieComponent ref="chart" width={window.innerWidth} height={90} />
            </div>
        );
    }

    componentDidMount() {
        this.series = this.refs.chart.addTimeSeries({},{ strokeStyle: 'rgba(95, 255, 95, 1)', lineWidth: 1 });
        this.last_trigger = null;
        this.frameListener = this.onFrame.bind(this);
        this.context.botConnection.events.on('frame', this.frameListener);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('frame', this.frameListener);
    }

    onFrame(model) {
        // Most recent value
        var value = jp.query(model, this.props.value);
        // When the packet arrived (relevant data may or may not be new)
        var timestamp = jp.query(model, this.props.timestamp);
        // Trigger for updates (indicates that data has been refreshed)
        var trigger = jp.query(model, this.props.trigger);

console.log(model, value, this.props.value, timestamp, trigger);

        if (timestamp.length && value.length && trigger.length && trigger !== this.last_trigger) {
            this.last_trigger = trigger;
            this.series.append(timestamp, value[0]);
        }
    }

}

export default windowSize(BotChart);
