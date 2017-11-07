import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch, Redirect } from 'react-router';
import OverlayLayers from './OverlayLayers';
import OverlaySetup from './OverlaySetup';

export default (props) => {
    return <div>
        <Nav pills>
            <NavItem><NavLink to={`/overlay/layers`} activeClassName="active" tag={RRNavLink}> Layers </NavLink></NavItem>
            <NavItem><NavLink to={`/overlay/setup`} activeClassName="active" tag={RRNavLink}> Setup </NavLink></NavItem>
        </Nav>
        <Switch>
            <Route path="/overlay/layers" component={OverlayLayers} />
            <Route path="/overlay/setup" component={OverlaySetup} />
            <Redirect path="*" to="/overlay/layers" />
        </Switch>
    </div>;
};
