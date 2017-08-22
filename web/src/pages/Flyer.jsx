import React from 'react';
import { Badge, Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';
import { ConfigText } from '../Config';

import FlyerAnalog from './FlyerAnalog';

export default (props) => (
    <div>
        <Badge className="right" color="info">
            <ConfigText item="mode" />
        </Badge>

        <Nav pills>
            <NavItem>
                <NavLink to="/flyer/analog" activeClassName="active" tag={RRNavLink}> Analog </NavLink>
            </NavItem>
        </Nav>

        <Switch>
            <Route path="/flyer/analog" component={FlyerAnalog} />
        </Switch>
    </div>
);
