import React from 'react';
import { ConfigSlider } from '../Config';
import { Chart, Series } from '../BotChart';

export default (props) => {
    const gimbal_status_timestamp = (model) => model.gimbal_status.local_timestamp;

    return <div>

        <h4>Motor Current</h4>

        <h6>Error duration</h6>
        <Chart>
            <Series
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.current_error_duration }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
        </Chart>

        <h6>Oscillation detector</h6>
        <Chart>
            <Series
                noBounds strokeStyle='#e22'
                value={ (model) => model.config.message.ConfigIsCurrent.gimbal.current_osc_detector_threshold }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
            <Series
                strokeStyle='#a22'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.current_osc_detector[0] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
            <Series
                strokeStyle='#2a2'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.current_osc_detector[1] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
            <Series
                strokeStyle='#22a'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.current_osc_detector[2] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
        </Chart>

        <h6>Oscillation detector, decay rate</h6>
        <ConfigSlider item="gimbal.current_osc_detector_decay_rate" min="0.0" max="0.5" step="1e-4" />

        <h6>Oscillation detector, threshold</h6>
        <ConfigSlider item="gimbal.current_osc_detector_threshold" min="0.0" max="50000" step="10" />

        <h6>Peak detector</h6>
        <Chart>
            <Series
                noBounds strokeStyle='#e22'
                value={ (model) => model.config.message.ConfigIsCurrent.gimbal.current_peak_detector_threshold }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
            <Series
                strokeStyle='#a22'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.current_peak_detector[0] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
            <Series
                strokeStyle='#2a2'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.current_peak_detector[1] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
            <Series
                strokeStyle='#22a'
                value={ (model) => model.gimbal_status.message.GimbalControlStatus.current_peak_detector[2] }
                trigger={gimbal_status_timestamp} timestamp={gimbal_status_timestamp} />
        </Chart>

        <h6>Peak detector, decay rate</h6>
        <ConfigSlider item="gimbal.current_peak_detector_decay_rate" min="0.0" max="0.5" step="1e-4" />

        <h6>Peak detector, update rate</h6>
        <ConfigSlider item="gimbal.current_peak_detector_update_rate" min="0.0" max="1.0" step="1e-4" />

        <h6>Peak detector, threshold</h6>
        <ConfigSlider item="gimbal.current_peak_detector_threshold" min="0.0" max="32767" step="1" />

        <h4>Limits</h4>

        <h6>Yaw limits, min/max</h6>
        <ConfigSlider item="gimbal.yaw_limits.0" min="-2048" max="2048" step="1" />
        <ConfigSlider item="gimbal.yaw_limits.1" min="-2048" max="2048" step="1" />

        <h6>Pitch limits, min/max</h6>
        <ConfigSlider item="gimbal.pitch_limits.0" min="-1024" max="1024" step="1" />
        <ConfigSlider item="gimbal.pitch_limits.1" min="-1024" max="1024" step="1" />

        <h6>Limiter slowdown extent</h6>
        <ConfigSlider item="gimbal.limiter_slowdown_extent.0" min="1" max="500" step="1e-4" />
        <ConfigSlider item="gimbal.limiter_slowdown_extent.1" min="1" max="500" step="1e-4" />

        <h4>Controller</h4>

        <h6>Hold proportional gain</h6>
        <ConfigSlider item="gimbal.hold_p_gain" min="0" max="0.5" step="1e-4" />

        <h6>Hold integral gain</h6>
        <ConfigSlider item="gimbal.hold_i_gain" min="0" max="0.0005" step="1e-8" />

        <h6>Limiter gain</h6>
        <ConfigSlider item="gimbal.limiter_gain" min="0" max="2.0" step="1e-4" />

        <h6>Re-home gain</h6>
        <ConfigSlider item="gimbal.rehome_gain" min="0" max="2.0" step="1e-4" />

        <h6>Max gimbal control rate</h6>
        <ConfigSlider item="gimbal.max_rate" min="0" max="800" step="1" />

        <h6>Tracking integrator decay rate</h6>
        <ConfigSlider item="gimbal.tracking_i_decay_rate" min="0" max="0.01" step="1e-4" />

        <h6>Hold integrator decay rate</h6>
        <ConfigSlider item="gimbal.hold_i_decay_rate" min="0" max="0.01" step="1e-4" />

        <h4>Yaw Tracking</h4>

        <h5>Gain region 0</h5>

        <h6>Width</h6>
        <ConfigSlider item="gimbal.yaw_gains.0.width" min="0" max="1.0" step="1e-4" />

        <h6>Proportional gain</h6>
        <ConfigSlider item="gimbal.yaw_gains.0.p_gain" min="0" max="2000.0" step="1e-4" />

        <h6>Integral gain</h6>
        <ConfigSlider item="gimbal.yaw_gains.0.i_gain" min="0" max="10.0" step="1e-4" />

        <h5>Gain region 1</h5>

        <h6>Width</h6>
        <ConfigSlider item="gimbal.yaw_gains.1.width" min="0" max="1.0" step="1e-4" />

        <h6>Proportional gain</h6>
        <ConfigSlider item="gimbal.yaw_gains.1.p_gain" min="0" max="2000.0" step="1e-4" />

        <h6>Integral gain</h6>
        <ConfigSlider item="gimbal.yaw_gains.1.i_gain" min="0" max="10.0" step="1e-4" />

        <h4>Pitch Tracking</h4>

        <h5>Gain region 0</h5>

        <h6>Width</h6>
        <ConfigSlider item="gimbal.pitch_gains.0.width" min="0" max="1.0" step="1e-4" />

        <h6>Proportional gain</h6>
        <ConfigSlider item="gimbal.pitch_gains.0.p_gain" min="0" max="2000.0" step="1e-4" />

        <h6>Integral gain</h6>
        <ConfigSlider item="gimbal.pitch_gains.0.i_gain" min="0" max="10.0" step="1e-4" />

        <h5>Gain region 1</h5>

        <h6>Width</h6>
        <ConfigSlider item="gimbal.pitch_gains.1.width" min="0" max="1.0" step="1e-4" />

        <h6>Proportional gain</h6>
        <ConfigSlider item="gimbal.pitch_gains.1.p_gain" min="0" max="2000.0" step="1e-4" />

        <h6>Integral gain</h6>
        <ConfigSlider item="gimbal.pitch_gains.1.i_gain" min="0" max="10.0" step="1e-4" />

        <h4>Firmware Configuration</h4>

        <h6>Scale for gyro feedback (gain)</h6>
        <ConfigSlider item="gimbal.values.98.0" min="0" max="1000" step="1" />
        <ConfigSlider item="gimbal.values.98.1" min="0" max="1000" step="1" />
        <ConfigSlider item="gimbal.values.98.2" min="0" max="1000" step="1" />

        <h6>Calibration point 0 (center)</h6>
        <ConfigSlider item="gimbal.values.77.0" min="0" max="4095" step="1" />
        <ConfigSlider item="gimbal.values.77.1" min="0" max="4095" step="1" />
        <ConfigSlider item="gimbal.values.77.2" min="0" max="4095" step="1" />

        <h6>Calibration point 1 (motor)</h6>
        <ConfigSlider item="gimbal.values.100.0" min="0" max="4095" step="1" />
        <ConfigSlider item="gimbal.values.100.1" min="0" max="4095" step="1" />
        <ConfigSlider item="gimbal.values.100.2" min="0" max="4095" step="1" />

    </div>;
}
