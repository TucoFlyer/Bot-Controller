# Bot-Controller

Central controller software for the Tuco Flyer bot.

* Runs on a desktop PC, alongside the video streaming software
* Communicates with each of the microcontroller boards via Ethernet
* Direct control with an Xbox 360 joystick
* Mobile control from a web dashboard
* WebSockets API

## Getting going

* Config file TBD, setup currently done via code
* Building the web app:

		cd web
		npm install
		npm run build

* Or for live web app development, use `npm start`
* Build and start the Bot-Controller process:

		cargo run

* The on-screen QR code and URL as well as `connection.txt` will have the key necessary to use the UI in authenticated mode.
