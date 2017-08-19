import React from 'react';
import { Button, ButtonToolbar } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Chart, Series } from '../BotChart';

export default (props) => {
    const id = parseInt(props.match.params.winchId, 10);
    const force_trigger = (model) => model.winches[id].message.WinchStatus[1].sensors.force.counter;
    const tick_trigger = (model) => model.winches[id].message.WinchStatus[1].tick_counter;
    const winch_timestamp = (model) => model.winches[id].local_timestamp;

    return <div>

        <h6>Force feedback</h6>
        <Chart height="150">
            <Series
                fullDataRate
                strokeStyle='#bbb'
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.measure }
                trigger={force_trigger} timestamp={winch_timestamp} />
            <Series
                fullDataRate
                strokeStyle='#71b1b3'
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.force.filtered }
                trigger={force_trigger} timestamp={winch_timestamp} />
            <Series
                noBounds strokeStyle='#b8383d'
                value={ (model) => model.winches[id].message.WinchStatus[1].command.force_min }
                trigger={force_trigger} timestamp={winch_timestamp} />
            <Series
                noBounds strokeStyle='#b8383d'
                value={ (model) => model.winches[id].message.WinchStatus[1].command.force_max }
                trigger={force_trigger} timestamp={winch_timestamp} />
        </Chart>

        <h6>Position feedback</h6>
        <Chart>
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].sensors.position }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>

        <h6>Velocity target</h6>
        <Chart>
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].command.velocity_target }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>

        <h6>PWM command</h6>
        <Chart>
            <Series
                value={ (model) => model.winches[id].message.WinchStatus[1].motor.pwm }
                trigger={tick_trigger} timestamp={winch_timestamp} />
        </Chart>


        <ButtonToolbar>
            <Button to={`/winch/${id}/timing`} color="info" tag={RRNavLink}> Timing check </Button>
        </ButtonToolbar>
    </div>
}