import React from 'react';
import { Chart, Series } from '../BotChart';

var colors = [ '#95172f', '#951776', '#641795', '#172095', '#179195', '#17953b', '#919517', '#955017' ]
var analog_trigger = (model) => model.flyer.message.FlyerSensors.analog.counter;
var flyer_timestamp = (model) => model.flyer.local_timestamp;

const FlyerAnalog = () => {
	var series = [];
	for (let id in colors) {
		series.push(<Series
			key={`flyer-analog-${id}`}
   			strokeStyle={colors[id]}
            value={ (model) => model.flyer.message.FlyerSensors.analog.values[id] }
	        trigger={analog_trigger} timestamp={flyer_timestamp}
	    />);
	}
	return <div>
	  	<Chart height="600">
	  		{ series }
	  	</Chart>
	</div>;
}

export default FlyerAnalog;
