import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { BotConnection } from '../BotConnection';
import Joystick from '../Joystick';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';
import { ConfigTextBlock } from '../Config';

import FlyerSensors from './FlyerSensors';
import Lighting from './Lighting';
import Vision from './Vision';
import Overlay from './Overlay';
import Gimbal from './Gimbal';

export default (props) => (
    <div>
        <Nav pills>
            <NavItem>
                <NavLink to="/flyer/sensors" activeClassName="active" tag={RRNavLink}> Sensors </NavLink>
            </NavItem>
            <NavItem>
                <NavLink to="/gimbal" activeClassName="active" tag={RRNavLink}> Gimbal </NavLink>
            </NavItem>
            <NavItem>
                <NavLink to="/vision" activeClassName="active" tag={RRNavLink}> Vision </NavLink>
            </NavItem>
            <NavItem>
                <NavLink to="/overlay" activeClassName="active" tag={RRNavLink}> Overlay </NavLink>
            </NavItem>
            <NavItem>
                <NavLink to="/lighting" activeClassName="active" tag={RRNavLink}> Lighting </NavLink>
            </NavItem>
        </Nav>

        <Switch>
            <Route path="/lighting" component={Lighting} />
            <Route path="/vision" component={Vision} />
            <Route path="/overlay" component={Overlay} />
            <Route path="/gimbal" component={Gimbal} />
            <Route path="/flyer/sensors" component={FlyerSensors} />
            <Route path="*" component={FlyerHome} />
        </Switch>
    </div>
);

class FlyerHome extends Component {
    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    render() {
        return <div>     

            <h6>Flyer Mode:</h6>
            <ConfigTextBlock item="mode" />

            <h6>Manual tracking control</h6>
            <Joystick
                onXY={ (x, y) => {
                    this.context.botConnection.send({ Command: { ManualControlValue: [ "CameraYaw", x ] }});
                    this.context.botConnection.send({ Command: { ManualControlValue: [ "CameraPitch", y ] }});
                }}
            />
        </div>;
    }
}

