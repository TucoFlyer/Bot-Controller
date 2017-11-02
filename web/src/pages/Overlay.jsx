import React from 'react';
import { ConfigSlider, ConfigColorAlpha } from '../Config';

export default (props) => {
    return <div>

        <h4>Tracking Region</h4>

        <h6>Thickness of outline</h6>
        <ConfigSlider item="overlay.tracked_region_outline_thickness" min="0.0" max="1.0" step="1e-4" />

        <h6>Color of outline in normal operation</h6>
        <ConfigColorAlpha item="overlay.tracked_region_default_color" />

        <h6>Color of outline during a manual movement</h6>
        <ConfigColorAlpha item="overlay.tracked_region_manual_color" />

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

        <h6>Gimbal tracking (gain) rectangle color</h6>
        <ConfigColorAlpha item="overlay.gimbal_tracking_rect_color" />

        <h6>Gimbal tracking (gain) rectangle display sensitivity</h6>
        <ConfigSlider item="overlay.gimbal_tracking_rect_sensitivity" min="0.0" max="100.0" step="1e-4" />

    </div>;
}
