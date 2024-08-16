# driverstation: A crate for controlling FRC robots

`driverstation` provides the means to control FRC robots without the use of the official NI DriverStationl.
You are able to enable and estop robots, change alliance stations, switch between Teleop, Auto, and Test modes,
transmit joysticks, and handle logs.

# Features

This crate is very much a work in progress,
so here's a list of things that need to be done and whether or not they are implemented:

- [x] Auto-connect to a simulation robot
- [ ] Auto-connect to a robot through a radio link
- [ ] Parse incoming UDP packets
- [ ] Parse incoming TCP packets
- [x] Construct outgoing UDP packets
- [x] Construct outgoing TCP packets
- [ ] Estop robot
- [ ] Enable/disable robot
- [ ] Change alliance station
- [ ] Switch robot mode
