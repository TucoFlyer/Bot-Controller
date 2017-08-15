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
        var series = this.refs.chart.addTimeSeries({},{ strokeStyle: 'rgba(95, 255, 95, 1)', lineWidth: 1 });
        this.last_trigger = null;

        this.messageListener = (messages) => {
            // Downsample potentially multiple messages to a single latest point, so we can avoid falling behind when CPU constrained

            var value_accum = 0;
            var value_count = 0;
            var latest_timestamp = null;

            for (var m of messages) {
                var timestamp = m.local_timestamp;
                var value = jp.query(m.message, this.props.path);
                var trigger_value = jp.query(m.message, this.props.trigger);
                if (value.length && trigger_value.length) {
                    if (trigger_value !== this.last_trigger) {
                        this.last_trigger = trigger_value;
                        value_accum += value[0];
                        value_count += 1;
                        latest_timestamp = timestamp;
                    }
                }
            }

            if (value_count > 0) {
                series.append(latest_timestamp, value_accum / value_count);
            }
        };

        this.context.botConnection.events.on('frame', this.messageListener);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('frame', this.messageListener);
    }
}

export default windowSize(BotChart);
