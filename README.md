# Bot-Controller

Central controller software for the Tuco Flyer bot.

* Runs on a desktop PC, alongside the video streaming software
* Communicates with each of the microcontroller boards via Ethernet
* WebSockets API
* Direct control with an Xbox 360 joystick
* Real-time mobile control via a web browser
* Optional sensor and metrics logging via InfluxDB

## Getting going

* Configuration via `config.yaml`
* Building the web app:

		cd web
		npm install
		npm run build

* Or for live web app development, use `npm start`
* Build and start the Bot-Controller process:

		cargo run

* The on-screen QR code and URL as well as `connection.txt` will have the key necessary to use the UI in authenticated mode.
