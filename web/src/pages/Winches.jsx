import React, { Component } from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';

import WinchSensors from './WinchSensors';
import WinchTiming from './WinchTiming';
import WinchPID from './WinchPID';

const Winch = (props) => {
    const id = parseInt(props.match.params.winchId, 10);
    return <div>
        <Nav pills>
            <NavItem><NavLink to={`/winch/${id}/sensors`} activeClassName="active" tag={RRNavLink}> Sensors </NavLink></NavItem>
            <NavItem><NavLink to={`/winch/${id}/pid`} activeClassName="active" tag={RRNavLink}> PID loop </NavLink></NavItem>
            <NavItem><NavLink to={`/winch/${id}/timing`} activeClassName="active" tag={RRNavLink}> Timing check </NavLink></NavItem>
        </Nav>
        <Switch>
            <Route path="/winch/:winchId/timing" component={WinchTiming} />
            <Route path="/winch/:winchId/pid" component={WinchPID} />
            <Route path="/winch/:winchId/sensors" component={WinchSensors} />
        </Switch>
    </div>;
}

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
                <Route path="/winch/:winchId" component={Winch} />
            </div>
        );
    }
}