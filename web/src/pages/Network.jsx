import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch, Redirect } from 'react-router';

import NetworkConsole from './NetworkConsole';
import NetworkModel from './NetworkModel';

export default (props) => (
    <Route path="/flyer">
        <div>
            <Nav pills>
                <NavItem>
                    <NavLink to="/net/model" activeClassName="active" tag={RRNavLink}> Model </NavLink>
                </NavItem>
                <NavItem>
                    <NavLink to="/net/console" activeClassName="active" tag={RRNavLink}> Console </NavLink>
                </NavItem>
            </Nav>
            <Switch>
                <Route path="/net/console" component={NetworkConsole} />
                <Route path="/net/model" component={NetworkModel} />
                <Redirect path="*" to="/net/model" />
            </Switch>
        </div>
    </Route>
);
