import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch, Redirect } from 'react-router';
import VisionStatus from './VisionStatus';
import VisionSetup from './VisionSetup';

export default (props) => {
    return <div>
        <Nav pills>
            <NavItem><NavLink to={`/vision/status`} activeClassName="active" tag={RRNavLink}> Status </NavLink></NavItem>
            <NavItem><NavLink to={`/vision/setup`} activeClassName="active" tag={RRNavLink}> Setup </NavLink></NavItem>
        </Nav>
        <Switch>
            <Route path="/vision/status" component={VisionStatus} />
            <Route path="/vision/setup" component={VisionSetup} />
            <Redirect path="*" to="/vision/status" />
        </Switch>
    </div>;
};
