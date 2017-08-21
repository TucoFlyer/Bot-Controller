import React from 'react';
import ReactDOM from 'react-dom';
import App from './App';
import { HashRouter } from 'react-router-dom';
import { BotConnection } from './BotConnection';
import 'bootstrap/dist/css/bootstrap.css';
import 'react-json-pretty/src/JSONPretty.adventure_time.css';
import './index.css';

const root = (
    <BotConnection>
        <HashRouter>
            <App/>
        </HashRouter>
    </BotConnection>
);

ReactDOM.render(root, document.getElementById('root'));
