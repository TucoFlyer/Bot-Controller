import React from 'react';
import { Chart, Series } from '../BotChart';

const colors = [ '#95172f', '#951776', '#641795', '#172095', '#179195', '#17953b', '#919517', '#955017' ]
const flyer_timestamp = (model) => model.flyer.local_timestamp;

const AnalogCharts = (props) => {
    const analog_trigger = (model) => model.flyer.message.FlyerSensors.analog.counter;
    let charts = [];
    for (let id = 0; id < 8; id += 1) {
        charts.push(
            <Chart
                key={`flyer-analog-${id}`}
                minValue="0"
                maxValue="4096"
                >
                <Series
               
                    strokeStyle={colors[id]}
                    value={ (model) => model.flyer.message.FlyerSensors.analog.values[id] }
                    trigger={analog_trigger} timestamp={flyer_timestamp}
                    />
            </Chart>
        );
    }
    return <div>{ charts }</div>;
}

const LidarCharts = (props) => {
    let charts = [];
    for (let id = 0; id < 4; id += 1) {
        charts.push(
            <Chart key={`flyer-lidar-${id}`}>
                <Series
                    strokeStyle={colors[id]}
                    value={ (model) => model.flyer.message.FlyerSensors.lidar.ranges[id] }
                    trigger={ (model) => model.flyer.message.FlyerSensors.lidar.counters[id] }
                    timestamp={flyer_timestamp}
                    />
            </Chart>
        );
    }
    return <div>{ charts }</div>;
}


export default (props) => {
    return <div>

        <h6>Orientation quaternion</h6>
        <Chart>
            <Series
                strokeStyle='#222'
                value={ (model) => model.flyer.message.FlyerSensors.imu.quaternion[0] }
                trigger={ (model) => model.flyer.message.FlyerSensors.imu.counter }
                timestamp={ (model) => model.flyer.local_timestamp } />
            <Series
                strokeStyle='#a22'
                value={ (model) => model.flyer.message.FlyerSensors.imu.quaternion[1] }
                trigger={ (model) => model.flyer.message.FlyerSensors.imu.counter }
                timestamp={ (model) => model.flyer.local_timestamp } />
            <Series
                strokeStyle='#2a2'
                value={ (model) => model.flyer.message.FlyerSensors.imu.quaternion[2] }
                trigger={ (model) => model.flyer.message.FlyerSensors.imu.counter }
                timestamp={ (model) => model.flyer.local_timestamp } />
            <Series
                strokeStyle='#22a'
                value={ (model) => model.flyer.message.FlyerSensors.imu.quaternion[3] }
                trigger={ (model) => model.flyer.message.FlyerSensors.imu.counter }
                timestamp={ (model) => model.flyer.local_timestamp } />
        </Chart>

        <h6>Linear acceleration</h6>
        <Chart>
            <Series
                strokeStyle='#a22'
                value={ (model) => model.flyer.message.FlyerSensors.imu.linear_accel[0] }
                trigger={ (model) => model.flyer.message.FlyerSensors.imu.counter }
                timestamp={ (model) => model.flyer.local_timestamp } />
            <Series
                strokeStyle='#2a2'
                value={ (model) => model.flyer.message.FlyerSensors.imu.linear_accel[1] }
                trigger={ (model) => model.flyer.message.FlyerSensors.imu.counter }
                timestamp={ (model) => model.flyer.local_timestamp } />
            <Series
                strokeStyle='#22a'
                value={ (model) => model.flyer.message.FlyerSensors.imu.linear_accel[2] }
                trigger={ (model) => model.flyer.message.FlyerSensors.imu.counter }
                timestamp={ (model) => model.flyer.local_timestamp } />
        </Chart>

        <h6>IR short-range distance</h6>
        <AnalogCharts/>

        <h6>LIDAR long-range distance</h6>
        <LidarCharts/>

        <h6>X-Band motion average</h6>
        <Chart>
            <Series
                value={ (model) => model.flyer.message.FlyerSensors.xband.speed_measure }
                trigger={ (model) => model.flyer.message.FlyerSensors.xband.measure_count }
                timestamp={ (model) => model.flyer.local_timestamp } />
        </Chart>

    </div>;
}
