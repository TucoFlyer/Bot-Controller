import React from 'react';
import { IfAuthenticated } from '../BotConnection';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch, Redirect } from 'react-router';
import LightingSchemes from './LightingSchemes';
import LightingColors from './LightingColors';
import LightingParams from './LightingParams';

export default (props) => {
    return <IfAuthenticated><div>
        <Nav pills>
            <NavItem><NavLink to={`/lighting/schemes`} activeClassName="active" tag={RRNavLink}> Schemes </NavLink></NavItem>
            <NavItem><NavLink to={`/lighting/colors`} activeClassName="active" tag={RRNavLink}> Colors </NavLink></NavItem>
            <NavItem><NavLink to={`/lighting/params`} activeClassName="active" tag={RRNavLink}> Params </NavLink></NavItem>
        </Nav>
        <Switch>
            <Route path="/lighting/schemes" component={LightingSchemes} />
            <Route path="/lighting/colors" component={LightingColors} />
            <Route path="/lighting/params" component={LightingParams} />
            <Redirect path="*" to="/lighting/schemes" />
        </Switch>
    </div></IfAuthenticated>;
};
