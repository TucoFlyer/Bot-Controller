import React from 'react';
import WS from 'ws-share';
import SmoothieComponent from 'react-smoothie';

class Chart extends React.Component {

    render() {
        return <SmoothieComponent ref="chart" width={this.props.width} height={this.props.height} />;
    }

    componentDidMount() {
        this.ws = new WS(this.props.uri);

        this.ws.on('close', (e) => {
            this.state = 'closed';
        });
        this.ws.on('error', (e) => {
            this.state = 'error';
        });
        this.ws.on('message', (e) => {
            this.state = 'receiving';
            console.log(e);
        });

        // var chart = this.refs.chart;
        // chart.addTimeSeries(series[0], { strokeStyle: 'rgba(255, 96, 96, 1)', lineWidth: 3 });
        // chart.addTimeSeries(series[1], { strokeStyle: 'rgba(96, 255, 96, 1)', lineWidth: 3 });
        // chart.addTimeSeries(series[2], { strokeStyle: 'rgba(96, 96, 255, 1)', lineWidth: 3 });

        var ts1 = this.refs.chart.addTimeSeries({},{ strokeStyle: 'rgba(0, 255, 0, 1)', fillStyle: 'rgba(0, 255, 0, 0.2)', lineWidth: 4 });
        var ts2 = this.refs.chart.addTimeSeries({},{ strokeStyle: 'rgba(255, 0, 0, 1)', fillStyle: 'rgba(255, 0, 0, 0.2)', lineWidth: 4 });

    this.dataGenerator = setInterval(function() {
      var time = new Date().getTime();
      ts1.append(time, Math.random());
      ts2.append(time, Math.random());
    }, 500);

  }

  componentWillUnmount() {
    clearInterval(this.dataGenerator);
  }
}

export default Chart;
