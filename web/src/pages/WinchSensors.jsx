import React from 'react';
import { Chart, Series } from '../BotChart';

const WinchSensors = (props) => {
    const id = parseInt(props.match.params.winchId, 10);
    const force_trigger = (model) => model.winches[id].message.WinchStatus[1].sensors.force.counter;
    const tick_trigger = (model) => model.winches[id].message.WinchStatus[1].tick_counter;
    const winch_timestamp = (model) => model.winches[id].local_timestamp;

    return <div>

        <h6>Force feedback</h6>
        <Chart height="200">
            <Series
                strokeStyle='#bbb'
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.measure }
                trigger={force_trigger} timestamp={winch_timestamp} />
            <Series
                strokeStyle='#71b1b3'
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.filtered }
                trigger={force_trigger} timestamp={winch_timestamp} />
        </Chart>

        <h6>Force limits</h6>
        <Chart height="50">
            <Series
                strokeStyle='#71b1b3'
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.filtered }
                trigger={force_trigger} timestamp={winch_timestamp} />
            <Series
                strokeStyle='#822'
                value={ (model) => model.winches[id].message.WinchStatus[1].command.force_min }
                trigger={force_trigger} timestamp={winch_timestamp} />
            <Series
                strokeStyle='#822'
                value={ (model) => model.winches[id].message.WinchStatus[1].command.force_max }
                trigger={force_trigger} timestamp={winch_timestamp} />
        </Chart>


        <h6>Position feedback</h6>
        <Chart>
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.position }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>

        <h6>PWM command</h6>
        <Chart>
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].command.velocity_target }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>

    </div>
}

export default WinchSensors;