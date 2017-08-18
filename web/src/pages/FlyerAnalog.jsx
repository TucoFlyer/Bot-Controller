import React from 'react';
import { Chart, Series } from '../BotChart';

const colors = [ '#95172f', '#951776', '#641795', '#172095', '#179195', '#17953b', '#919517', '#955017' ]
const analog_trigger = (model) => model.flyer.message.FlyerSensors.analog.counter;
const flyer_timestamp = (model) => model.flyer.local_timestamp;

export default (props) => {
    let series = [];
    for (let id in colors) {
        series.push(<Series
            key={`flyer-analog-${id}`}
            strokeStyle={colors[id]}
            value={ (model) => model.flyer.message.FlyerSensors.analog.values[id] }
            trigger={analog_trigger} timestamp={flyer_timestamp}
        />);
    }
    return <div>
        <Chart height="360">
            { series }
        </Chart>
    </div>;
}
