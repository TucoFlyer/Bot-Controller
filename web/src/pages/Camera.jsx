import React from 'react';
import { ConfigSlider, ConfigColorAlpha } from '../Config';

export default (props) => {
    return <div>

        <h4>Video Overlay</h4>

        <h6>Debug text height</h6>
        <ConfigSlider item="overlay.debug_text_height" min="0.0" max="0.1" step="1e-4" />

        <h6>Debug text color</h6>
        <ConfigColorAlpha item="overlay.debug_color" />

        <h6>Detector outlines, minimum probability</h6>
        <ConfigSlider item="overlay.detector_outline_min_prob" min="0.0" max="1.0" step="1e-4" />

        <h6>Detector outlines, max thickness scale</h6>
        <ConfigSlider item="overlay.detector_outline_max_thickness" min="0.0" max="0.1" step="1e-4" />

        <h6>Detector default outline color</h6>
        <ConfigColorAlpha item="overlay.detector_default_outline_color" />

        <h6>Detector labels, minimum probability</h6>
        <ConfigSlider item="overlay.detector_label_min_prob" min="0.0" max="1.0" step="1e-4" />

        <h6>Label text size</h6>
        <ConfigSlider item="overlay.label_text_size" min="0.0" max="0.1" step="1e-4" />

        <h6>Label text color</h6>
        <ConfigColorAlpha item="overlay.label_color" />

        <h6>Label text background</h6>
        <ConfigColorAlpha item="overlay.label_background_color" />

        <h6>Tracked region, outline thickness</h6>
        <ConfigSlider item="overlay.tracked_region_outline_thickness" min="0.0" max="1.0" step="1e-4" />

        <h6>Tracked region, outline color</h6>
        <ConfigColorAlpha item="overlay.tracked_region_outline_color" />

        <h6>Border rect thickness</h6>
        <ConfigSlider item="overlay.border_thickness" min="0.0" max="1.0" step="1e-4" />

        <h6>Halt border color</h6>
        <ConfigColorAlpha item="overlay.halt_color" />

        <h6>Gimbal tracking (gain) rectangle color</h6>
        <ConfigColorAlpha item="overlay.gimbal_tracking_rect_color" />

        <h6>Gimbal tracking (gain) rectangle display sensitivity</h6>
        <ConfigSlider item="overlay.gimbal_tracking_rect_sensitivity" min="0.0" max="100.0" step="1e-4" />

        <h4>Computer Vision</h4>

        <h6>Manual control speeds, minimum (dead zone)</h6>
        <ConfigSlider item="vision.min_manual_control_speed" min="0.0" max="1.0" step="1e-4" />

        <h6>Manual control speeds, maximum</h6>
        <ConfigSlider item="vision.max_manual_control_speed" min="0.0" max="2.0" step="1e-4" />

        <h6>Tracked region, minimum PSR before discard</h6>
        <ConfigSlider item="vision.tracking_min_psr" min="0.0" max="20.0" step="1e-4" />

        <h6>Tracked region, minimum area before discard</h6>
        <ConfigSlider item="vision.tracking_min_area" min="0.0" max="0.5" step="1e-4" />

        <h6>Tracked region, maximum area before discard</h6>
        <ConfigSlider item="vision.tracking_max_area" min="0.0" max="3.5" step="1e-4" />

        <h6>Tracked region, default area at initialization</h6>
        <ConfigSlider item="vision.tracking_default_area" min="0.0" max="0.5" step="1e-4" />

    </div>;
}
