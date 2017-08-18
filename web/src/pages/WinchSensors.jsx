import React from 'react';
import { Chart, Series } from '../BotChart';

const WinchSensors = (props) => {
    const id = parseInt(props.match.params.winchId, 10);
    return <div>

        <h6>Force feedback</h6>
        <Chart>
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.measure }
                trigger={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.counter }
                timestamp={ (model) => model.winches[id].local_timestamp } />
            <Series
                strokeStyle='#71b1b3'
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.filtered }
                trigger={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.counter }
                timestamp={ (model) => model.winches[id].local_timestamp } />
        </Chart>


        <h6>Position feedback</h6>
        <Chart>
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.position }
                trigger={ (model) => model.winches[id].message.WinchStatus[1].tick_counter }
                timestamp={ (model) => model.winches[id].local_timestamp } />
        </Chart>

        <h6>PWM command</h6>
        <Chart>
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].command.velocity_target }
                trigger={ (model) => model.winches[id].message.WinchStatus[1].tick_counter }
                timestamp={ (model) => model.winches[id].local_timestamp } />
        </Chart>

    </div>
}

export default WinchSensors;