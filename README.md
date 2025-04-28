blimp_ground_ws_interface
=========================
Websocket protocol used between components of our blimp ground station system. Those are:

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
 