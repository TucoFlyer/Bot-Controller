import React from 'react';
import { ConfigSlider } from '../Config';

export default (props) => {
    return <div>

        <h4>General parameters</h4>

        <h6>Flash rate (Hz)</h6>
        <ConfigSlider item="lighting.current.flash_rate_hz" min="0" max="5.0" step="1e-4" />

        <h6>Exponent to shape flash animation</h6>
        <ConfigSlider item="lighting.current.flash_exponent" min="0" max="8.0" step="1e-4" />

        <h4>Flyer</h4>

        <h6>Brightness of saucer section</h6>
        <ConfigSlider item="lighting.current.flyer_saucer_brightness" min="0" max="4" step="0.1" />

        <h6>Z scale</h6>
        <ConfigSlider item="lighting.current.flyer_z_scale" min="0" max="2" step="0.01" />

        <h6>Ring size</h6>
        <ConfigSlider item="lighting.current.flyer_ring_size" min="0" max="4" step="0.1" />

        <h6>Ring thickness</h6>
        <ConfigSlider item="lighting.current.flyer_ring_thickness" min="0" max="4" step="0.1" />

        <h6>Dot size</h6>
        <ConfigSlider item="lighting.current.flyer_dot_size" min="0" max="4" step="0.1" />

        <h6>Dot pattern rate</h6>
        <ConfigSlider item="lighting.current.flyer_dot_pattern_rate" min="0" max="100" step="0.1" />

        <h6>Dot pattern smoothness</h6>
        <ConfigSlider item="lighting.current.flyer_dot_pattern_smoothness" min="0" max="1" step="1e-4" />

        <h4>Winch animation</h4>

        <h6>Spatial length of wave animation (m)</h6>
        <ConfigSlider item="lighting.current.winch_wavelength_m" min="0.01" max="0.2" step="1e-4" />

        <h6>Spatial length of wave window (m)</h6>
        <ConfigSlider item="lighting.current.winch_wave_window_length_m" min="0.01" max="0.4" step="1e-4" />

        <h6>Peak amplitude of wave animation</h6>
        <ConfigSlider item="lighting.current.winch_wave_amplitude" min="0.0" max="2.0" step="1e-4" />

        <h6>Exponent to shape wave animation</h6>
        <ConfigSlider item="lighting.current.winch_wave_exponent" min="0.0" max="10.0" step="1e-4" />

        <h6>Filtered speed at which wave animation reaches full amplitude (m/s)</h6>
        <ConfigSlider item="lighting.current.winch_speed_for_full_wave_amplitude_m_per_sec" min="0.0" max="0.2" step="1e-4" />

        <h6>Filter parameter for speed used above</h6>
        <ConfigSlider item="lighting.current.winch_velocity_filter_param" min="0.0" max="0.1" step="1e-4" />

    </div>;
}

