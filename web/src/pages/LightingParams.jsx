import React from 'react';
import { ConfigSlider } from '../Config';

export default (props) => {
    return <div>

        <h4>General parameters</h4>

        <h6>Flash rate (Hz)</h6>
        <ConfigSlider item="lighting.current.flash_rate_hz" min="0" max="5.0" step="1e-2" />

        <h6>Exponent to shape flash animation</h6>
        <ConfigSlider item="lighting.current.flash_exponent" min="0" max="8.0" step="1e-2" />

        <h4>Winch animation</h4>

        <h6>Spatial length of wave animation (m)</h6>
        <ConfigSlider item="lighting.current.winch.wavelength_m" min="0.01" max="0.5" step="1e-2" />

        <h6>Spatial length of wave window (m)</h6>
        <ConfigSlider item="lighting.current.winch.wave_window_length_m" min="0.01" max="0.4" step="1e-2" />

        <h6>Peak amplitude of wave animation</h6>
        <ConfigSlider item="lighting.current.winch.wave_amplitude" min="0.0" max="2.0" step="1e-2" />

        <h6>Exponent to shape wave animation</h6>
        <ConfigSlider item="lighting.current.winch.wave_exponent" min="0.0" max="10.0" step="1e-2" />

        <h6>Filtered speed at which wave animation reaches full amplitude (m/s)</h6>
        <ConfigSlider item="lighting.current.winch.speed_for_full_wave_amplitude_m_per_sec" min="0.0" max="0.2" step="1e-2" />

        <h6>Filter parameter for speed used above</h6>
        <ConfigSlider item="lighting.current.winch.velocity_filter_param" min="0.0" max="0.1" step="1e-4" />

    </div>;
}

