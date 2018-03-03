import React from 'react';
import { ConfigColor } from '../Config';

export default (props) => {
    return <div>

        <h4>Flyer ring colors</h4>

        <h6>Background color</h6>
        <ConfigColor item="lighting.current.flyer_ring_background_color" />

        <h6>Dot color</h6>
        <ConfigColor item="lighting.current.flyer_dot_color" />

        <h4>Winch colors</h4>

        <h6>Normal-mode background color</h6>
        <ConfigColor item="lighting.current.winch_normal_color" />

        <h6>Manual-mode selection color</h6>
        <ConfigColor item="lighting.current.winch_manual_selected_color" />

        <h6>Manual-mode background color</h6>
        <ConfigColor item="lighting.current.winch_manual_deselected_color" />

        <h6>Halt-mode background color</h6>
        <ConfigColor item="lighting.current.winch_halt_color" />

        <h6>Error background color</h6>
        <ConfigColor item="lighting.current.winch_error_color" />

        <h6>Flashing color when stuck</h6>
        <ConfigColor item="lighting.current.winch_stuck_color" />

        <h6>Wave color for commanded position</h6>
        <ConfigColor item="lighting.current.winch_command_color" />

        <h6>Wave color for sensed motion</h6>
        <ConfigColor item="lighting.current.winch_motion_color" />

    </div>;
}
