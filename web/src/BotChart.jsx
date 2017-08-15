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
        var path = this.props.path;
        var series = this.refs.chart.addTimeSeries({},{ strokeStyle: 'rgba(95, 255, 95, 1)', lineWidth: 1 });

        this.messageListener = (msg) => {
            var value = jp.query(msg, path);
            if (value.length) {
                series.append(msg.timestamp, value[0]);
            }
        };

        this.context.botConnection.events.on('message', this.messageListener);
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('message', this.messageListener);
    }
}

export default windowSize(BotChart);
