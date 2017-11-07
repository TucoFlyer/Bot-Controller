import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch, Redirect } from 'react-router';
import GimbalStatus from './GimbalStatus';
import GimbalSetup from './GimbalSetup';
import GimbalInternals from './GimbalInternals';
import './Gimbal.css';

export default (props) => {
    return <div>
        <Nav pills>
            <NavItem><NavLink to={`/gimbal/status`} activeClassName="active" tag={RRNavLink}> Status </NavLink></NavItem>
            <NavItem><NavLink to={`/gimbal/setup`} activeClassName="active" tag={RRNavLink}> Setup </NavLink></NavItem>
            <NavItem><NavLink to={`/gimbal/internals`} activeClassName="active" tag={RRNavLink}> Internals </NavLink></NavItem>
        </Nav>
        <Switch>
            <Route path="/gimbal/status" component={GimbalStatus} />
            <Route path="/gimbal/setup" component={GimbalSetup} />
            <Route path="/gimbal/internals" component={GimbalInternals} />
            <Redirect path="*" to="/gimbal/status" />
        </Switch>
    </div>;
};
