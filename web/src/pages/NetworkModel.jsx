import React, { Component } from 'react';
import BotJSON from '../BotJSON';

export default class NetworkModel extends Component {
    render() {
        return <div>

            <h2>Flyer</h2>

            <div className="NetworkModel">
                <BotJSON value={ (model) => model.flyer } />
            </div>

            <h2>Winches</h2>

            <div className="NetworkModel">
                <BotJSON value={ (model) => model.winches } />
            </div>

            <h2>Camera</h2>

            <div className="NetworkModel">
                <BotJSON value={ (model) => model.camera } />
            </div>

        </div>;
    }
}
