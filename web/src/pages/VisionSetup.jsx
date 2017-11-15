import React from 'react';
import { ConfigSlider } from '../Config';

export default (props) => {
    return <div>

        <h4>Manual Controls</h4>

        <h6>Deadzone size</h6>
        <ConfigSlider item="vision.manual_control_deadzone" min="0.0" max="0.5" step="1e-4" />

        <h6>Speed multiplier</h6>
        <ConfigSlider item="vision.manual_control_speed" min="0.0" max="6.0" step="1e-4" />

        <h6>Restoring force</h6>
        <ConfigSlider item="vision.manual_control_restoring_force" min="0.0" max="10.0" step="1e-4" />

        <h6>Idle timeout (seconds)</h6>
        <ConfigSlider item="vision.manual_control_timeout_sec" min="0.0" max="0.1" step="1e-4" />

        <h4>Tracked Region</h4>

        <h6>Default area for manual control</h6>
        <ConfigSlider item="vision.tracking_default_area" min="0.0" max="2.0" step="1e-4" />

        <h6>Minimum allowed area during automatic control</h6>
        <ConfigSlider item="vision.tracking_min_area" min="0.0" max="2.0" step="1e-4" />

        <h6>Maximum allowed area during automatic control</h6>
        <ConfigSlider item="vision.tracking_max_area" min="0.0" max="2.0" step="1e-4" />

    </div>;
}
