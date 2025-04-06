blimp_ground_ws_interface
=========================
Definition of the websocket protocol used between components of our blimp ground station system. Those are:

Servers:
 - `blimp_ground_d`
 - `blimp_simulator`

Clients:
 - `blimp_yoke_control`
 - `blimp_viz`

Capabilities
------------
 - Subscribing to parameters of the blimp's state and receiving their live updates,
 - Sending control commands to the blimp

Currently, this crate holds the struct definitions only. Implementations **should** permit communication in both JSON and Postcard format, recognised by the server according to the first received packet.
 