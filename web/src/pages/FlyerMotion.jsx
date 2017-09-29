import React from 'react';
import { Chart, Series } from '../BotChart';

export default (props) => {
    return <div>

        <h6>X-Band motion average</h6>
        <Chart>
            <Series
                value={ (model) => model.flyer.message.FlyerSensors.xband.speed_measure }
                trigger={ (model) => model.flyer.message.FlyerSensors.xband.measure_count }
                timestamp={ (model) => model.flyer.local_timestamp } />
        </Chart>

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

    </div>;
}
