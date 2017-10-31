import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';
import { ConfigTextBlock } from '../Config';
import { IfAuthenticated } from '../BotConnection';

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

const FlyerHome = (props) => (
    <div>

        <h5>Flyer Mode:</h5>
        <ConfigTextBlock item="mode" />

    </div>
);
