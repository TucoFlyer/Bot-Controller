import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';
import { ConfigTextBlock } from '../Config';
import { IfAuthenticated } from '../BotConnection';

import FlyerAnalog from './FlyerAnalog';
import Lighting from './Lighting';

export default (props) => (
    <div>
        <Nav pills>
            <IfAuthenticated><NavItem>
                <NavLink to="/lighting" activeClassName="active" tag={RRNavLink}> Lighting </NavLink>
            </NavItem></IfAuthenticated>
            <NavItem>
                <NavLink to="/flyer/analog" activeClassName="active" tag={RRNavLink}> Analog </NavLink>
            </NavItem>
        </Nav>

        <Switch>
            <Route path="/lighting" component={Lighting} />
            <Route path="/flyer/analog" component={FlyerAnalog} />
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
