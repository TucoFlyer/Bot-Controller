import React from 'react';
import { Chart, Series } from '../BotChart';

export default (props) => {
    const id = parseInt(props.match.params.winchId, 10);
    const tick_trigger = (model) => model.winches[id].message.WinchStatus[1].tick_counter;
    const winch_timestamp = (model) => model.winches[id].local_timestamp;

    return <div>

        <h6>Calibrating winch #{id}</h6>

    </div>
}
