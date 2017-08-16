import React from 'react';
import BotChart from './BotChart';
import BotConnection from './BotConnection';
import './App.css';

export default class App extends React.Component {
    render() {
        return <div className="App">
            <BotConnection>

                <h1>Tuco Flyer</h1>

                <h2>Winch force feedback</h2>
                <BotChart
                    value="winches[0].message.WinchStatus[1].sensors.force.measure"
                    trigger="winches[0].message.WinchStatus[1].sensors.force.counter"
                    timestamp="winches[0].local_timestamp" />

                <h2>Winch position feedback</h2>
                <BotChart
                    value="winches[0].message.WinchStatus[1].sensors.position"
                    trigger="winches[0].message.WinchStatus[1].tick_counter"
                    timestamp="winches[0].local_timestamp" />

                <h2>Winch PWM command</h2>
                <BotChart
                    value="winches[0].message.WinchStatus[1].command.velocity_target"
                    trigger="winches[0].message.WinchStatus[1].tick_counter"
                    timestamp="winches[0].local_timestamp" />

            </BotConnection>
        </div>;
    }
}
