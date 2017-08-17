import React, { Component } from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';

import './App.css';
import Home from './pages/Home';
import FlyerAnalog from './pages/FlyerAnalog';
import WinchSensors from './pages/WinchSensors';


export default class App extends Component {

    constructor(props) {
        super(props);
        this.state = {
            winchList: [0, 1, 2, 3],
        };
    }

    render() {
        return (
            <div className="App">
                <div className="AppLogo">
                    <img src="/tuco-flyer.png" alt="Tuco Flyer logo" />
                </div>

                <Nav pills>
                    <NavItem>
                        <NavLink exact to="/" activeClassName="active" tag={RRNavLink}> Home </NavLink>
                    </NavItem>
                    <NavItem>
                        <NavLink to="/flyer" activeClassName="active" tag={RRNavLink}> Flyer </NavLink>
                    </NavItem>
                    <NavItem>
                        <NavLink to="/winch" activeClassName="active" tag={RRNavLink}> Winch </NavLink>
                    </NavItem>
                </Nav>
    			<Switch>

    				<Route path="/flyer">
                        <div>
                            <Nav pills>
                                <NavItem>
                                    <NavLink to="/flyer/analog" activeClassName="active" tag={RRNavLink}> Analog </NavLink>
                                </NavItem>
                            </Nav>
                            <Switch>
                                <Route path="/flyer/analog" component={FlyerAnalog} />
                            </Switch>
                        </div>
                    </Route>

                    <Route path="/winch">
                        <div>
                            <Nav pills>
                                {this.state.winchList.map(function (id) { return (
                                    <NavItem key={`winch-${id}`}>
                                        <NavLink to={`/winch/${id}`} activeClassName="active" tag={RRNavLink}> Bot {id} </NavLink>
                                    </NavItem>
                                )})}
                            </Nav>
                            <Switch>
                                <Route path="/winch/:winchId" component={WinchSensors} />
                            </Switch>
                        </div>
                    </Route>

                    <Route path="/" component={Home} />

    			</Switch>

            </div>
        );
    }
}
