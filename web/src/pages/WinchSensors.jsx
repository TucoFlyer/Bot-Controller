import React from 'react';
import { Chart, Series } from '../BotChart';

const WinchSensors = (props) => {
    var winchId = parseInt(props.match.params.winchId, 10);
    return <div>

        <h5>Force feedback</h5>
        <Chart>
            <Series
                value={ (model) => model.winches[winchId].message.WinchStatus[1].sensors.force.measure }
                trigger={ (model) => model.winches[winchId].message.WinchStatus[1].sensors.force.counter }
                timestamp={ (model) => model.winches[winchId].local_timestamp } />
        </Chart>

        <h5>Position feedback</h5>
        <Chart>
            <Series
                value={ (model) => model.winches[winchId].message.WinchStatus[1].sensors.position }
                trigger={ (model) => model.winches[winchId].message.WinchStatus[1].tick_counter }
                timestamp={ (model) => model.winches[winchId].local_timestamp } />
        </Chart>

        <h5>PWM command</h5>
        <Chart>
            <Series
                value={ (model) => model.winches[winchId].message.WinchStatus[1].command.velocity_target }
                trigger={ (model) => model.winches[winchId].message.WinchStatus[1].tick_counter }
                timestamp={ (model) => model.winches[winchId].local_timestamp } />
        </Chart>

    </div>
}

export default WinchSensors;