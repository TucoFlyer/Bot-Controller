import React from 'react';
import { ConfigColor } from '../Config';

export default (props) => {
    return <div>

        <h4>Winch colors</h4>

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

    </div>;
}
