import React from 'react';
import { Chart, Series } from '../BotChart';

export default (props) => {
    const id = parseInt(props.match.params.winchId, 10);
    const tick_trigger = (model) => model.winches[id].message.WinchStatus[1].tick_counter;
    const winch_timestamp = (model) => model.winches[id].local_timestamp;

    return <div>
        <h6>Timing check, tick# vs. timestamp</h6>
        <Chart millisPerPixel="2">
            <Series
                fullDataRate
                strokeStyle="#f00"
                value={ (model) => ( tick_trigger(model) / 8 % 1 ) }
                trigger={tick_trigger} timestamp={winch_timestamp} />

            <Series
                fullDataRate
                strokeStyle="#00f"
                value={ (model) => (winch_timestamp(model) / 1000 % 1) }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>

    </div>
}
