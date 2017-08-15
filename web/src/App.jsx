import React from 'react';
import BotChart from './BotChart';
import BotConnection from './BotConnection';
import './App.css';

export default class App extends React.Component {
    render() {
        return <div className="App">
            <BotConnection>

                <h1>Tuco Flyer</h1>

{/*
                <h2>Some analog</h2>
                <BotChart path="FlyerSensors.analog.values[0]" trigger="FlyerSensors.analog.counter" />
                <BotChart path="FlyerSensors.analog.values[1]" trigger="FlyerSensors.analog.counter" />
*/}

                <h2>Winch force feedback</h2>
                <BotChart path="WinchStatus[1].sensors.force.measure" trigger="WinchStatus[1].sensors.force.counter" />
                <h2>Winch position feedback</h2>
                <BotChart path="WinchStatus[1].sensors.position" trigger="WinchStatus[1].tick_counter" />
                <h2>Winch velocity feedback</h2>
                <BotChart path="WinchStatus[1].sensors.velocity" trigger="WinchStatus[1].tick_counter" />
                <h2>Winch PWM command</h2>
                <BotChart path="WinchStatus[1].command.velocity_target" trigger="WinchStatus[1].tick_counter" />

            </BotConnection>
        </div>;
    }
}
