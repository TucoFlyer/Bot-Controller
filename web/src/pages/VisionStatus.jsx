import React from 'react';
import { Chart, Series } from '../BotChart';

function rect_left(r) { return r[0] }
function rect_top(r) { return r[1] }
function rect_right(r) { return r[0]+r[2] }
function rect_bottom(r) { return r[1]+r[3] }
function rect_area(r) { return r[2]*r[3] }

function region_trigger(model) { return model.camera.region_tracking.message.Command.CameraRegionTracking.frame }
function region_timestamp(model) { return model.camera.region_tracking.local_timestamp }

export default (props) => {
    return <div>

        <h6>Correlation tracker rectangle edges</h6>
        <Chart>
            <Series
                strokeStyle='#a22'
                value={ (model) => rect_left(model.camera.region_tracking.message.Command.CameraRegionTracking.rect) }
                trigger={region_trigger} timestamp={region_timestamp} />
            <Series
                strokeStyle='#a22'
                value={ (model) => rect_right(model.camera.region_tracking.message.Command.CameraRegionTracking.rect) }
                trigger={region_trigger} timestamp={region_timestamp} />
            <Series
                strokeStyle='#22a'
                value={ (model) => rect_top(model.camera.region_tracking.message.Command.CameraRegionTracking.rect) }
                trigger={region_trigger} timestamp={region_timestamp} />
            <Series
                strokeStyle='#22a'
                value={ (model) => rect_bottom(model.camera.region_tracking.message.Command.CameraRegionTracking.rect) }
                trigger={region_trigger} timestamp={region_timestamp} />
        </Chart>

        <h6>Correlation tracker rectangle area</h6>
        <Chart>
            <Series
                strokeStyle='#a22'
                value={ (model) => rect_area(model.camera.region_tracking.message.Command.CameraRegionTracking.rect) }
                trigger={region_trigger} timestamp={region_timestamp} />
        </Chart>

        <h6>Correlation tracker quality (Peak to Side Ratio)</h6>
        <Chart>
            <Series
                value={ (model) => model.camera.region_tracking.message.Command.CameraRegionTracking.psr }
                trigger={region_trigger} timestamp={region_timestamp} />
        </Chart>

        <h6>Object detector runtime (milliseconds) </h6>
        <Chart>
            <Series
                value={ (model) => model.camera.object_detection.message.Command.CameraObjectDetection.detector_nsec * 1e-6 }
                trigger={ (model) => model.camera.object_detection.message.Command.CameraObjectDetection.frame }
                timestamp={ (model) => model.camera.object_detection.local_timestamp } />
        </Chart>

        <h6>Correlation tracker runtime (milliseconds) </h6>
        <Chart>
            <Series
                value={ (model) => model.camera.region_tracking.message.Command.CameraRegionTracking.tracker_nsec * 1e-6 }
                trigger={region_trigger} timestamp={region_timestamp} />
        </Chart>

    </div>;
}
