import React, { Component } from 'react';
import { ConfigSlider } from '../Config';
import { IfAuthenticated } from '../BotConnection';

export default class extends Component {
    render () {
        return <div>

            <IfAuthenticated><div>
                <h6>RGB brightness scale</h6>
                <ConfigSlider item="params.led_rgb_brightness_scale.0" min="0" max="2.0" step="1e-2" />
                <ConfigSlider item="params.led_rgb_brightness_scale.1" min="0" max="2.0" step="1e-2" />
                <ConfigSlider item="params.led_rgb_brightness_scale.2" min="0" max="2.0" step="1e-2" />
            </div></IfAuthenticated>

        </div>;
    }
}

