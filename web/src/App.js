import React, { Component } from 'react';
import Chart from './Chart';
import './App.css';

class App extends Component {
  render() {
    return (
      <div className="App">
        <h2>Heyo flufftown</h2>
        <Chart width="100%" height="32px" uri="ws://10.0.0.2"/>
        <Chart/>
        <Chart/>
        <Chart/>
      </div>
    );
  }
}

export default App;
