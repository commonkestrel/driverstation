# driverstation: A crate for controlling FRC robots

`driverstation` provides the means to control FRC robots without the use of the official NI DriverStationl.
You are able to enable and estop robots, change alliance stations, switch between Teleop, Auto, and Test modes,
transmit joysticks, and handle logs.

# Features

This crate is very much a work in progress,
so here's a list of things that need to be done and whether or not they are implemented:

- [x] Auto-connect to a simulation robot
- [x] Auto-connect to a robot through an ethernet link
- [ ] Auto-connect to a robot through a radio link
- [x] Parse incoming UDP packets
- [ ] Parse incoming TCP packets
- [x] Construct outgoing UDP packets
- [x] Construct outgoing TCP packets
- [x] Estop robot
- [x] Enable/disable robot
- [x] Change alliance station
- [x] Switch robot mode
