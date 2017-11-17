import React from 'react';
import { ConfigSlider } from '../Config';

export default (props) => {
    return <div>

        <h6>Particle swarm</h6>
        <ConfigSlider item="overlay.particle_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Raw tracking rectangle</h6>
        <ConfigSlider item="overlay.tracked_region_default_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Manual moves to tracking rectangle</h6>
        <ConfigSlider item="overlay.tracked_region_manual_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Active gain regions</h6>
        <ConfigSlider item="overlay.gain_region_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Object detector outlines</h6>
        <ConfigSlider item="overlay.detector_default_outline_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Label text</h6>
        <ConfigSlider item="overlay.label_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Label background</h6>
        <ConfigSlider item="overlay.label_background_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Debug text</h6>
        <ConfigSlider item="overlay.debug_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Debug text background</h6>
        <ConfigSlider item="overlay.debug_background_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Gimbal background</h6>
        <ConfigSlider item="overlay.gimbal_background_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Gimbal outline</h6>
        <ConfigSlider item="overlay.gimbal_outline_color.3" min="0.0" max="1.0" step="1e-2" />

        <h6>Gimbal cursor</h6>
        <ConfigSlider item="overlay.gimbal_cursor_color.3" min="0.0" max="1.0" step="1e-2" />

    </div>;
}
