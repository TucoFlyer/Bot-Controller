import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';

import NetworkConsole from './NetworkConsole';

export default (props) => (
    <Route path="/flyer">
        <div>
            <Nav pills>
                <NavItem>
                    <NavLink to="/net/console" activeClassName="active" tag={RRNavLink}> Console </NavLink>
                </NavItem>
            </Nav>
            <Switch>
                <Route path="/net/console" component={NetworkConsole} />
            </Switch>
        </div>
    </Route>
);
