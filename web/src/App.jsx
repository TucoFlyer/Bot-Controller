import React from 'react';
import BotChart from './BotChart';
import BotConnection from './BotConnection';
import './App.css';

export default class App extends React.Component {
    render() {
        return <div className="App">
            <BotConnection>

                <h1>Tuco Flyer</h1>

                <h2>Some analog</h2>
                <BotChart path="FlyerSensors.analog.values[0]"/>
                <BotChart path="FlyerSensors.analog.values[1]"/>

                <h2>Winch force feedback</h2>
                <BotChart path="WinchStatus[1].sensors.force.measure"/>
                <h2>Winch position feedback</h2>
                <BotChart path="WinchStatus[1].sensors.position"/>
                <h2>Winch velocity feedback</h2>
                <BotChart path="WinchStatus[1].sensors.velocity"/>
                <h2>Winch PWM command</h2>
                <BotChart path="WinchStatus[1].command.velocity_target"/>

            </BotConnection>
        </div>;
    }
}
