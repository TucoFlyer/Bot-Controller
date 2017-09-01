import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';

import './App.css';
import AuthStatus from './AuthStatus';
import Bot from './pages/Bot';
import Winches from './pages/Winches';
import Network from './pages/Network';

export default () => (
    <div className="App">
        <div className="AppLogo">
            <img src="/tuco-flyer.png" alt="Tuco Flyer logo" />
        </div>

        <AuthStatus className="right" />

        <Nav pills>
            <NavItem><NavLink to="/" exact activeClassName="active" tag={RRNavLink}> Bot </NavLink></NavItem>
            <NavItem><NavLink to="/winch" activeClassName="active" tag={RRNavLink}> Winch </NavLink></NavItem>
            <NavItem><NavLink to="/net" activeClassName="active" tag={RRNavLink}> Net </NavLink></NavItem>
        </Nav>

    	<Switch>
            <Route path="/winch" component={Winches} />
            <Route path="/net" component={Network} />
            <Route path="/" component={Bot} />
    	</Switch>
    </div>
);
