import React, { Component } from 'react';
import { ConfigSlider, ConfigColor } from '../Config';
import { IfAuthenticated } from '../BotConnection';

export default class extends Component {
    render () {
        return <IfAuthenticated><div>

                <h4>Shared Settings</h4>

                <h6>Brightness</h6>
                <ConfigSlider item="lighting.current.brightness" min="0" max="2.0" step="1e-2" />

                <h6>Flash rate (Hz)</h6>
                <ConfigSlider item="lighting.current.flash_rate_hz" min="0" max="5.0" step="1e-2" />

                <h6>Exponent to shape flash animation</h6>
                <ConfigSlider item="lighting.current.flash_exponent" min="0" max="10.0" step="1e-2" />

                <h4>Winch Lighting</h4>

                <h6>Normal-mode background color</h6>
                <ConfigColor item="lighting.current.winch.normal_color" />

                <h6>Manual-mode background color</h6>
                <ConfigColor item="lighting.current.winch.manual_color" />

                <h6>Halt-mode background color</h6>
                <ConfigColor item="lighting.current.winch.halt_color" />

                <h6>Error background color</h6>
                <ConfigColor item="lighting.current.winch.error_color" />

                <h6>Flashing color when stuck</h6>
                <ConfigColor item="lighting.current.winch.stuck_color" />

                <h6>Wave color for commanded position</h6>
                <ConfigColor item="lighting.current.winch.command_color" />

                <h6>Wave color for sensed motion</h6>
                <ConfigColor item="lighting.current.winch.motion_color" />

                <h6>Spatial length of wave animation (m)</h6>
                <ConfigSlider item="lighting.current.winch.wavelength_m" min="0.01" max="0.5" step="1e-2" />

                <h6>Peak amplitude of wave animation</h6>
                <ConfigSlider item="lighting.current.winch.wave_amplitude" min="0.0" max="2.0" step="1e-2" />

                <h6>Exponent to shape wave animation</h6>
                <ConfigSlider item="lighting.current.winch.wave_exponent" min="0.0" max="40.0" step="1e-2" />

                <h6>Filtered speed at which wave animation reaches full amplitude (m/s)</h6>
                <ConfigSlider item="lighting.current.winch.speed_for_full_wave_amplitude_m_per_sec" min="0.0" max="0.75" step="1e-2" />

                <h6>Filter parameter for speed used above</h6>
                <ConfigSlider item="lighting.current.winch.velocity_filter_param" min="0.0" max="0.2" step="1e-2" />

        </div></IfAuthenticated>;
    }
}

