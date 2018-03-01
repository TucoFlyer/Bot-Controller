import React from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';

import './App.css';
import AuthStatus from './AuthStatus';
import Bot from './pages/Bot';
import Winches from './pages/Winches';
import Network from './pages/Network';
import FlyerSensors from './pages/FlyerSensors';
import Lighting from './pages/Lighting';
import Vision from './pages/Vision';
import Video from './pages/Video';
import Overlay from './pages/Overlay';
import Gimbal from './pages/Gimbal';

export default () => (
    <div className="App">
        <div className="AppLogo">
            <img src="/tuco-flyer.png" alt="Tuco Flyer logo" />
        </div>

        <AuthStatus className="right" />

        <Nav pills>
            <NavItem><NavLink to="/" exact activeClassName="active" tag={RRNavLink}> Bot </NavLink></NavItem>
            <NavItem><NavLink to="/video" exact activeClassName="active" tag={RRNavLink}> Video </NavLink></NavItem>
            <NavItem><NavLink to="/gimbal" activeClassName="active" tag={RRNavLink}> Gimbal </NavLink></NavItem>
            <NavItem><NavLink to="/winch" activeClassName="active" tag={RRNavLink}> Winch </NavLink></NavItem>
            <NavItem><NavLink to="/overlay" activeClassName="active" tag={RRNavLink}> Overlay </NavLink></NavItem>
            <NavItem><NavLink to="/flyer/sensors" activeClassName="active" tag={RRNavLink}> Sensors </NavLink></NavItem>
            <NavItem><NavLink to="/vision" activeClassName="active" tag={RRNavLink}> Vision </NavLink></NavItem>
            <NavItem><NavLink to="/lighting" activeClassName="active" tag={RRNavLink}> Lights </NavLink></NavItem>
            <NavItem><NavLink to="/net" activeClassName="active" tag={RRNavLink}> Net </NavLink></NavItem>
        </Nav>

        <hr/>

    	<Switch>
            <Route path="/winch" component={Winches} />
            <Route path="/net" component={Network} />
            <Route path="/lighting" component={Lighting} />
            <Route path="/vision" component={Vision} />
            <Route path="/video" component={Video} />
            <Route path="/overlay" component={Overlay} />
            <Route path="/gimbal" component={Gimbal} />
            <Route path="/flyer/sensors" component={FlyerSensors} />
            <Route path="/" component={Bot} />
    	</Switch>

        <hr/>

    </div>
);
