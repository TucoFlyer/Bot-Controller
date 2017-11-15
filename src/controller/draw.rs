use message::*;
use vecmath::*;
use config::{Config, ControllerMode};
use overlay::DrawingContext;
use controller::manual::ManualControls;

pub fn mode_indicator(config: &Config, draw: &mut DrawingContext) {
    if config.mode == ControllerMode::Halted {
        draw.current.outline_color = config.overlay.halt_color;
        draw.current.outline_thickness = config.overlay.border_thickness;
        draw.outline_rect(rect_offset(config.vision.border_rect, -config.overlay.border_thickness));
    }
}

pub fn debug_text(config: &Config, draw: &mut DrawingContext, debug: String)
{
    draw.current.color = config.overlay.debug_color;
    draw.current.text_height = config.overlay.debug_text_height;
    draw.current.background_color = config.overlay.debug_background_color;
    draw.current.outline_thickness = 0.0;
    draw.text(rect_topleft(config.vision.border_rect), [0.0, 0.0], &debug).unwrap();
}

pub fn detected_objects(config: &Config, draw: &mut DrawingContext, detected: &CameraDetectedObjects) {
    draw.current.outline_color = config.overlay.detector_default_outline_color;
    for obj in &detected.objects {
        if obj.prob >= config.overlay.detector_outline_min_prob {
            draw.current.outline_thickness = obj.prob * config.overlay.detector_outline_max_thickness;
            draw.outline_rect(obj.rect);
            let outer_rect = rect_offset(obj.rect, draw.current.outline_thickness);

            if obj.prob >= config.overlay.detector_label_min_prob {
                draw.current.text_height = config.overlay.label_text_size;
                draw.current.color = config.overlay.label_color;
                draw.current.background_color = config.overlay.label_background_color;
                draw.current.outline_thickness = 0.0;

                let label = if config.overlay.detector_label_prob_values {
                    format!("{} p={:.3}", obj.label, obj.prob)
                } else {
                    obj.label.clone()
                };

                draw.text(rect_topleft(outer_rect), [0.0, 1.0], &label).unwrap();
            }
        }
    }
}

pub fn tracking_rect(config: &Config, draw: &mut DrawingContext, tracked: &CameraTrackedRegion, manual: &ManualControls) {
    if !tracked.is_empty() {
        draw.current.outline_thickness = config.overlay.tracked_region_outline_thickness;

        if manual.camera_control_active() {
            draw.current.outline_color = config.overlay.tracked_region_manual_color;
            draw.outline_rect(tracked.rect);

        } else {
            draw.current.outline_color = config.overlay.tracked_region_default_color;
            draw.outline_rect(tracked.rect);

            let outer_rect = rect_offset(tracked.rect, config.overlay.tracked_region_outline_thickness);

            let tr_label = format!("psr={:.2} age={} area={:.3}", tracked.psr, tracked.age, rect_area(tracked.rect));

            draw.current.text_height = config.overlay.label_text_size;
            draw.current.color = config.overlay.label_color;
            draw.current.background_color = config.overlay.label_background_color;
            draw.current.outline_thickness = 0.0;
            draw.text(rect_bottomleft(outer_rect), [0.0, 0.0], &tr_label).unwrap();
        }
    }
}

pub fn tracking_gains(config: &Config, draw: &mut DrawingContext, gimbal: &Option<GimbalControlStatus>) {
    if let &Some(ref gimbal) = gimbal {
        draw.current.color = config.overlay.gain_region_color;
        let border = config.vision.border_rect;

        for index in 0 .. config.gimbal.yaw_gains.len() {
            let gain = &config.gimbal.yaw_gains[index];
            let activation = gimbal.yaw_gain_activations[index];
            if activation < 0.0 {
                let width = -activation;
                draw.solid_rect([ rect_right(border) - gain.width, border[1], width, border[3] ]);
            } else if activation > 0.0 {
                let width = activation;
                draw.solid_rect([ rect_left(border) + gain.width - width, border[1], width, border[3] ]);
            }
        }

        for index in 0 .. config.gimbal.pitch_gains.len() {
            let gain = &config.gimbal.pitch_gains[index];
            let activation = gimbal.pitch_gain_activations[index];
            if activation < 0.0 {
                let width = -activation;
                draw.solid_rect([ border[0], rect_bottom(border) - gain.width, border[2], width ]);
            } else if activation > 0.0 {
                let width = activation;
                draw.solid_rect([ border[0], rect_top(border) + gain.width - width, border[2], width ]);
            }
        }
    }
}
