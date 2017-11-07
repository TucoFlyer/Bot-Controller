import React from 'react';
import { ConfigSlider } from '../Config';

export default (props) => {
    return <div>

        <h4>Tracking</h4>

        <h6>Drift compensation</h6>
        <ConfigSlider item="gimbal.drift_compensation.0" min="-5" max="5" step="1e-4" />
        <ConfigSlider item="gimbal.drift_compensation.1" min="-5" max="5" step="1e-4" />

        <h6>Integral decay rate</h6>
        <ConfigSlider item="gimbal.integral_decay_rate" min="0" max="0.1" step="1e-4" />

        <h5>Yaw gain region 0</h5>

        <h6>Width</h6>
        <ConfigSlider item="gimbal.yaw_gains.0.width" min="0" max="1.0" step="1e-4" />

        <h6>Proportional gain</h6>
        <ConfigSlider item="gimbal.yaw_gains.0.p_gain" min="0" max="1000.0" step="1e-4" />

        <h6>Integral gain</h6>
        <ConfigSlider item="gimbal.yaw_gains.0.i_gain" min="0" max="1000.0" step="1e-4" />

        <h5>Yaw gain region 1</h5>

        <h6>Width</h6>
        <ConfigSlider item="gimbal.yaw_gains.1.width" min="0" max="1.0" step="1e-4" />

        <h6>Proportional gain</h6>
        <ConfigSlider item="gimbal.yaw_gains.1.p_gain" min="0" max="1000.0" step="1e-4" />

        <h6>Integral gain</h6>
        <ConfigSlider item="gimbal.yaw_gains.1.i_gain" min="0" max="1000.0" step="1e-4" />

        <h5>Pitch gain region 0</h5>

        <h6>Width</h6>
        <ConfigSlider item="gimbal.pitch_gains.0.width" min="0" max="1.0" step="1e-4" />

        <h6>Proportional gain</h6>
        <ConfigSlider item="gimbal.pitch_gains.0.p_gain" min="0" max="1000.0" step="1e-4" />

        <h6>Integral gain</h6>
        <ConfigSlider item="gimbal.pitch_gains.0.i_gain" min="0" max="1000.0" step="1e-4" />

        <h5>Pitch gain region 1</h5>

        <h6>Width</h6>
        <ConfigSlider item="gimbal.pitch_gains.1.width" min="0" max="1.0" step="1e-4" />

        <h6>Proportional gain</h6>
        <ConfigSlider item="gimbal.pitch_gains.1.p_gain" min="0" max="1000.0" step="1e-4" />

        <h6>Integral gain</h6>
        <ConfigSlider item="gimbal.pitch_gains.1.i_gain" min="0" max="1000.0" step="1e-4" />

        <h4>Limits</h4>

        <h6>Yaw limits, min/max</h6>
        <ConfigSlider item="gimbal.yaw_limits.0" min="-2048" max="2048" step="1" />
        <ConfigSlider item="gimbal.yaw_limits.1" min="-2048" max="2048" step="1" />

        <h6>Pitch limits, min/max</h6>
        <ConfigSlider item="gimbal.pitch_limits.0" min="-1024" max="1024" step="1" />
        <ConfigSlider item="gimbal.pitch_limits.1" min="-1024" max="1024" step="1" />

        <h6>Limiter gain</h6>
        <ConfigSlider item="gimbal.limiter_gain" min="0" max="10" step="1e-4" />

        <h6>Limiter slowdown extent</h6>
        <ConfigSlider item="gimbal.limiter_slowdown_extent" min="1" max="500" step="1e-4" />

        <h6>Max gimbal control rate</h6>
        <ConfigSlider item="gimbal.max_rate" min="0" max="1500" step="1" />

    </div>;
}
