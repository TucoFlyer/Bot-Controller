import React, { Component } from 'react';
import JSONPretty from 'react-json-pretty';
import PropTypes from 'prop-types';
import { Button } from 'reactstrap';
import { CustomPicker } from 'react-color';
import { Saturation, Hue } from 'react-color/lib/components/common'
import { BotConnection } from './BotConnection';
import reactCSS from 'reactcss';
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
    for (let part of pathArray(path)) {
        if (config === undefined) {
            break;
        }
        config = config[part];
    }
    return config;
}

function pathArray(str_or_list) {
    let array = Array.from(str_or_list.split ? str_or_list.split(".") : str_or_list);
    for (let i in array) {
        const str_or_int = array[i];
        const intval = parseInt(str_or_int, 10);
        array[i] = Number.isNaN(intval) ? str_or_int : intval;
    }
    return array;
}

// Inverse of getByPath, creates intermediate nodes as needed
export const setByPath = function(config, path, item) {
    let parts = pathArray(path);
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

export const forceToKg = function(model, winchId, counts) {
    const cal = model.config.message.ConfigIsCurrent.winches[winchId].calibration;
    return (counts - cal.force_zero_count) * cal.kg_force_per_count;
}

export const distToMeters = function(model, winchId, counts) {
    const cal = model.config.message.ConfigIsCurrent.winches[winchId].calibration;
    return counts * cal.m_dist_per_count;
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
        const str = JSON.stringify(value);
        return <JSONPretty {...props} json={str} />;
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

// Color picker for a vector config item
export const ConfigColor = withConfig(class extends Component {
    render() {
        let { config, item } = this.props;
        const value = getByPath(config, item);
        return <ConfigColorPicker
            color={{
                r: value[0] * 255,
                g: value[1] * 255,
                b: value[2] * 255
            }}
            onChange={this.handleChange.bind(this)}
        />
    }

    static contextTypes = {
        botConnection: PropTypes.instanceOf(BotConnection),
    }

    handleChange(color, event) {
       this.context.botConnection.socket.send(JSON.stringify({
            UpdateConfig: setByPath({}, this.props.item, [
                color.rgb.r / 255.0,
                color.rgb.g / 255.0,
                color.rgb.b / 255.0,
            ])
        }));
    }
});


const ConfigColorPicker = CustomPicker(function(props) {
    let styles = reactCSS({
        'default': {
            preview: {
                background: props.hex
            }
        }
    });
    return <div className="ConfigColorPicker">
        <div className="saturation">
            <Saturation
                hsl={ props.hsl }
                hsv={ props.hsv }
                onChange={ props.onChange }
            />
        </div>
        <div className="hue">
            <Hue
                hsl={ props.hsl }
                onChange={ props.onChange }
            />
        </div>
        <div className="preview" style={styles.preview} ></div>
    </div>;
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
