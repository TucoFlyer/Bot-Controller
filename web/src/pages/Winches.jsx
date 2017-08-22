import React, { Component } from 'react';
import { Nav, NavItem, NavLink } from 'reactstrap';
import { NavLink as RRNavLink } from 'react-router-dom';
import { Route, Switch } from 'react-router';
import { IfAuthenticated, withConfig } from '../BotConnection';

import WinchSensors from './WinchSensors';
import WinchTiming from './WinchTiming';
import WinchPID from './WinchPID';
import WinchControl from './WinchControl';

const Winch = (props) => {
    const id = parseInt(props.match.params.winchId, 10);
    return <div>
        <Nav pills>
            <IfAuthenticated><NavItem><NavLink to={`/winch/${id}/control`} activeClassName="active" tag={RRNavLink}> Control </NavLink></NavItem></IfAuthenticated>
            <NavItem><NavLink to={`/winch/${id}/sensors`} activeClassName="active" tag={RRNavLink}> Sensors </NavLink></NavItem>
            <NavItem><NavLink to={`/winch/${id}/pid`} activeClassName="active" tag={RRNavLink}> PID </NavLink></NavItem>
            <NavItem><NavLink to={`/winch/${id}/timing`} activeClassName="active" tag={RRNavLink}> Timing </NavLink></NavItem>
        </Nav>
        <Switch>
            <Route path="/winch/:winchId/control" component={WinchControl} />
            <Route path="/winch/:winchId/timing" component={WinchTiming} />
            <Route path="/winch/:winchId/pid" component={WinchPID} />
            <Route path="/winch/:winchId/sensors" component={WinchSensors} />
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
                    { winchNav }
                </Nav>
                <Route path="/winch/:winchId" component={Winch} />
            </div>
        );
    }
});
