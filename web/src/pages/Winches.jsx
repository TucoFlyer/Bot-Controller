import React, { Component } from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch, Redirect } from 'react-router';
import { IfAuthenticated } from '../BotConnection';
import { ConfigText, withConfig } from '../Config';
import { Badge } from 'reactstrap';

import WinchAll from './WinchAll';
import WinchSensors from './WinchSensors';
import WinchTiming from './WinchTiming';
import WinchPID from './WinchPID';
import WinchControl from './WinchControl';
import WinchCalibrator from './WinchCalibrator';

const Winch = (props) => {
    const id = parseInt(props.match.params.winchId, 10);
    if (!(id >= 0)) {
        return null;  // NaN or negative
    }

    return <div>
        <Badge className="right" color="secondary">
            <ConfigText item={["winches", id, "addr"]} />
        </Badge>
        <Nav pills>
            <IfAuthenticated><NavItem><NavLink to={`/winch/${id}/control`} activeClassName="active" tag={RRNavLink}> Control </NavLink></NavItem></IfAuthenticated>
            <NavItem><NavLink to={`/winch/${id}/sensors`} activeClassName="active" tag={RRNavLink}> Sensors </NavLink></NavItem>
            <NavItem><NavLink to={`/winch/${id}/pid`} activeClassName="active" tag={RRNavLink}> PID </NavLink></NavItem>
        </Nav>
        <Switch>
            <Route path="/winch/:winchId/cal" component={WinchCalibrator} />
            <Route path="/winch/:winchId/control" component={WinchControl} />
            <Route path="/winch/:winchId/pid" component={WinchPID} />
            <Route path="/winch/:winchId/sensors" component={WinchSensors} />
            <Route path="/winch/:winchId/timing" component={WinchTiming} />
            <Redirect path="*" to={`/winch/${id}/sensors`} />
        </Switch>
    </div>;
};

export default withConfig( class extends Component {
    render() {
        var winchNav = [];

        for (let id in this.props.config.winches) {
            winchNav.push(
                <NavItem key={`winch-${id}`}>
                    <NavLink to={`/winch/${id}`} activeClassName="active" tag={RRNavLink}> Bot {id} </NavLink>
                </NavItem>
            );
        }

        return (
            <div>
                <Nav pills>
                    <NavItem>
                        <NavLink to={`/winch/all`} activeClassName="active" tag={RRNavLink}> All </NavLink>
                    </NavItem>
                    { winchNav }
                </Nav>
                <Route path="/winch/all" component={WinchAll} />
                <Route path="/winch/:winchId" component={Winch} />
                <Redirect path="*" to="/winch/all" />
            </div>
        );
    }
});
