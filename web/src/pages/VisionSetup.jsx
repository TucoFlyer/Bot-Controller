import React from 'react';
import { ConfigSlider } from '../Config';

export default (props) => {
    return <div>

        <h6>Manual control deadzone size</h6>
        <ConfigSlider item="vision.manual_control_deadzone" min="0.0" max="0.5" step="1e-4" />

        <h6>Manual control speed multiplier</h6>
        <ConfigSlider item="vision.manual_control_speed" min="0.0" max="6.0" step="1e-4" />

        <h6>Manual control restoring force</h6>
        <ConfigSlider item="vision.manual_control_restoring_force" min="0.0" max="10.0" step="1e-4" />

        <h6>Manual control timeout (seconds)</h6>
        <ConfigSlider item="vision.manual_control_timeout_sec" min="0.0" max="2.0" step="1e-4" />

        <h6>Tracked region, default area for manual control</h6>
        <ConfigSlider item="vision.tracking_default_area" min="0.0" max="0.5" step="1e-4" />

    </div>;
}
