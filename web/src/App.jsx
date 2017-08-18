import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';

import './App.css';
import Home from './pages/Home';
import Flyer from './pages/Flyer';
import Winches from './pages/Winches';

export default () => (
    <div className="App">
        <div className="AppLogo">
            <img src="/tuco-flyer.png" alt="Tuco Flyer logo" />
        </div>

        <Nav pills>
            <NavItem><NavLink exact to="/" activeClassName="active" tag={RRNavLink}> Home </NavLink></NavItem>
            <NavItem><NavLink to="/flyer" activeClassName="active" tag={RRNavLink}> Flyer </NavLink></NavItem>
            <NavItem><NavLink to="/winch" activeClassName="active" tag={RRNavLink}> Winch </NavLink></NavItem>
        </Nav>
    	<Switch>
            <Route path="/winch" component={Winches} />
            <Route path="/flyer" component={Flyer} />
            <Route path="/" component={Home} />
    	</Switch>
    </div>
);
