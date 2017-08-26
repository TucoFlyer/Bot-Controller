import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';
import { ConfigTextBlock } from '../Config';

import FlyerAnalog from './FlyerAnalog';

export default (props) => (
    <div>
        <Nav pills>
            <NavItem>
                <NavLink to="/flyer/analog" activeClassName="active" tag={RRNavLink}> Analog </NavLink>
            </NavItem>
        </Nav>

        <Switch>
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
