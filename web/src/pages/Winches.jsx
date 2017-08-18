import React, { Component } from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';

import WinchSensors from './WinchSensors';
import WinchTiming from './WinchTiming';

export default class Winches extends Component {

    constructor(props) {
        super(props);
        this.state = {
            winchList: [0, 1, 2, 3],
        };
    }

    render() {
        return (
            <div>
                <Nav pills>
                    { this.state.winchList.map(function (id) { return (
                        <NavItem key={`winch-${id}`}>
                            <NavLink to={`/winch/${id}`} activeClassName="active" tag={RRNavLink}> Bot {id} </NavLink>
                        </NavItem>
                    )})}
                </Nav>
                <Switch>
                    <Route path="/winch/:winchId/timing" component={WinchTiming} />
                    <Route path="/winch/:winchId" component={WinchSensors} />
                </Switch>
            </div>
        );
    }
}