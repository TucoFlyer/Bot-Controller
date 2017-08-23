import React, { Component } from 'react';
import JSONPretty from 'react-json-pretty';
import PropTypes from 'prop-types';
import { Button } from 'reactstrap';
import { BotConnection } from './BotConnection';
import './Config.css';

// Higher order component that adds the configuration as a prop on the wrapped component
export const withConfig = (ComposedComponent, options) => class extends Component {

    constructor() {
        super();
        this.state = { config: null };
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    handleConfig = (event) => {
        this.setState({ config: event.message.ConfigIsCurrent });
        if (options && options.once) {
            this.context.botConnection.events.removeListener('config', this.handleConfig);            
        }
    }

    componentDidMount() {
        this.context.botConnection.events.on('config', this.handleConfig);
        if (this.context.botConnection.model.config) {
            this.handleConfig(this.context.botConnection.model.config);
        }
    }

    componentWillUnmount() {
        this.context.botConnection.events.removeListener('config', this.handleConfig);
    }

    render() {
        if (this.state.config !== null) {
            return <ComposedComponent {...this.props} config={this.state.config} />;
        } else {
            return null;
        }
    }
};

// Path is something like "foo.bar.blah" or ["foo", 5, "blah"]
export const getByPath = function(config, path) {
    for (let part of path.split ? path.split(".") : path) {
        if (config === undefined) {
            break;
        }
        config = config[part];
    }
    return config;
}

// Inverse of getByPath, creates intermediate nodes as needed
export const setByPath = function(config, path, item) {
    let parts = Array.from(path.split ? path.split(".") : path);
    let obj = config;
    for (let i = 0; i < parts.length - 1; i++) {
        if (parts[i] in obj) {
            obj = obj[parts[i]];
        } else if (typeof parts[i+1] === 'number') {
            obj = obj[parts[i]] = [];
        } else {
            obj = obj[parts[i]] = {};
        }
    }
    obj[parts.pop()] = item;
    return config;
}

// Text span displaying a config item
export const ConfigText = withConfig(class extends Component {
    render() {
        let { config, item, ...props } = this.props;
        const value = getByPath(config, item);
        const str = typeof(value) === 'string' ? value : JSON.stringify(value);
        return <span {...props}> { str } </span>;
    }
});

// JSON text box displaying a config item
export const ConfigTextBlock = withConfig(class extends Component {
    render() {
        let { config, item, ...props } = this.props;
        const value = getByPath(config, item);
        return <JSONPretty {...props} json={value} />;
    }
});

// Slider to edit a numeric config item
export const ConfigSlider = withConfig(class extends Component {
    render() {
        let { config, item, ...props } = this.props;
        const value = getByPath(config, item);
        return (
            <div className="ConfigSlider">
                <input
                    {...props}
                    type="range"
                    value={value}
                    onChange={this.handleChange.bind(this)}
                />
                <ConfigText item={item} />
            </div>
        );
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    handleChange(event) {
        this.context.botConnection.socket.send(JSON.stringify({
            UpdateConfig: setByPath({}, this.props.item, parseFloat(event.target.value))
        }));
    }
});

// Button that stores the first config it gets, click to revert
export const ConfigRevertButton = withConfig(class extends Component {
    render() {
        let { config, item, ...props } = this.props;
        const value = getByPath(config, item);
        return <Button {...props} onClick={this.handleClick}>
            Revert to { value }
        </Button>;
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    handleClick = (event) => {
        const item = this.props.item;
        const config = setByPath({}, item, getByPath(this.props.config, item));
        this.context.botConnection.socket.send(JSON.stringify({ UpdateConfig: config }));        
    }
},{
    once: true,
});
