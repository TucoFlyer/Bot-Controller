import React from 'react';
import { ConfigSlider, ConfigColorAlpha } from '../Config';

export default (props) => {
    return <div>

        <h4>Particle swarm</h4>

        <h6>Particle color and opacity</h6>
        <ConfigColorAlpha item="overlay.particle_color" />

        <h6>Size of each particle edge</h6>
        <ConfigSlider item="overlay.particle_size" min="0.0" max="0.2" step="1e-4" />

        <h6>Number of particles</h6>
        <ConfigSlider item="overlay.particle_count" min="0.0" max="200" step="1" />

        <h6>Damping</h6>
        <ConfigSlider item="overlay.particle_damping" min="0.0" max="0.1" step="1e-6" />

        <h6>Gain for snapping to tracking rectangle edge</h6>
        <ConfigSlider item="overlay.particle_edge_gain" min="0.0" max="1.0" step="1e-6" />

        <h6>Gain for perpendicular motion</h6>
        <ConfigSlider item="overlay.particle_perpendicular_gain" min="-0.3" max="0.3" step="1e-6" />

        <h6>Gain for random motion</h6>
        <ConfigSlider item="overlay.particle_random_gain" min="0" max="2" step="1e-6" />

        <h6>Minimum separation distance</h6>
        <ConfigSlider item="overlay.particle_min_distance" min="0.0" max="0.5" step="1e-6" />

        <h6>Gain for minimum distance</h6>
        <ConfigSlider item="overlay.particle_min_distance_gain" min="0.0" max="0.1" step="1e-6" />

        <h4>Tracking Region</h4>

        <h6>Thickness of outline</h6>
        <ConfigSlider item="overlay.tracked_region_outline_thickness" min="0.0" max="1.0" step="1e-4" />

        <h6>Color of outline in normal operation</h6>
        <ConfigColorAlpha item="overlay.tracked_region_default_color" />

        <h6>Color of outline during a manual movement</h6>
        <ConfigColorAlpha item="overlay.tracked_region_manual_color" />

        <h6>Color of active gain regions</h6>
        <ConfigColorAlpha item="overlay.gain_region_color" />

        <h4>Object Detection</h4>

        <h6>Min probability for outlines or labels</h6>
        <ConfigSlider item="overlay.detector_outline_min_prob" min="0.0" max="1.0" step="1e-4" />

        <h6>Min probability for labels</h6>
        <ConfigSlider item="overlay.detector_label_min_prob" min="0.0" max="1.0" step="1e-4" />

        <h6>Thickness scale for outlines</h6>
        <ConfigSlider item="overlay.detector_outline_max_thickness" min="0.0" max="0.1" step="1e-4" />

        <h6>Color of outlines</h6>
        <ConfigColorAlpha item="overlay.detector_default_outline_color" />

        <h6>Label text size</h6>
        <ConfigSlider item="overlay.label_text_size" min="0.0" max="0.1" step="1e-4" />

        <h6>Label text color</h6>
        <ConfigColorAlpha item="overlay.label_color" />

        <h6>Label text background</h6>
        <ConfigColorAlpha item="overlay.label_background_color" />

        <h4>Debug Text</h4>

        <h6>Debug text height</h6>
        <ConfigSlider item="overlay.debug_text_height" min="0.0" max="0.2" step="1e-4" />

        <h6>Debug text color</h6>
        <ConfigColorAlpha item="overlay.debug_color" />

        <h6>Debug text background color</h6>
        <ConfigColorAlpha item="overlay.debug_background_color" />

        <h4>Status Rectangles</h4>

        <h6>Border rect thickness</h6>
        <ConfigSlider item="overlay.border_thickness" min="0.0" max="1.0" step="1e-4" />

        <h6>Halt border color</h6>
        <ConfigColorAlpha item="overlay.halt_color" />

        <h4>Gimbal Status</h4>

        <h6>Center position</h6>
        <ConfigSlider item="overlay.gimbal_rect_center.0" min="-1.0" max="1.0" step="1e-4" />
        <ConfigSlider item="overlay.gimbal_rect_center.1" min="-1.0" max="1.0" step="1e-4" />

        <h6>Width</h6>
        <ConfigSlider item="overlay.gimbal_rect_width" min="0" max="1.0" step="1e-4" />

        <h6>Background color</h6>
        <ConfigColorAlpha item="overlay.gimbal_background_color" />

        <h6>Outline color</h6>
        <ConfigColorAlpha item="overlay.gimbal_outline_color" />

        <h6>Outline thickness</h6>
        <ConfigSlider item="overlay.gimbal_outline_thickness" min="0.0" max="0.05" step="1e-4" />

        <h6>Cursor color</h6>
        <ConfigColorAlpha item="overlay.gimbal_cursor_color" />

        <h6>Cursor size</h6>
        <ConfigSlider item="overlay.gimbal_cursor_size" min="0.0" max="0.2" step="1e-4" />

    </div>;
}
