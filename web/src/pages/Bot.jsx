import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';
import { ConfigTextBlock } from '../Config';
import { IfAuthenticated } from '../BotConnection';

import FlyerSensors from './FlyerSensors';
import Lighting from './Lighting';

export default (props) => (
    <div>
        <Nav pills>
            <IfAuthenticated><NavItem>
                <NavLink to="/lighting" activeClassName="active" tag={RRNavLink}> Lighting </NavLink>
            </NavItem></IfAuthenticated>
            <NavItem>
                <NavLink to="/flyer/sensors" activeClassName="active" tag={RRNavLink}> Sensors </NavLink>
            </NavItem>
        </Nav>

        <Switch>
            <Route path="/lighting" component={Lighting} />
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
