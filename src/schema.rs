pub const PROTOCOL_ORGANIZATION: &str = "spacecoffee";
pub const PROTOCOL_PROJECT: &str = "blimp";
pub const PROTOCOL_VERSION: u16 = 1;

/// Messages sent by the server
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub enum MessageG2V {
    MotorSpeed { id: u8, speed: f32 },
    ServoPosition { id: u8, angle: f32 },
    SensorData { id: String, data: f64 },
    State(BlimpState),
}

/// Messages sent by the client
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub enum MessageV2G {
    DeclareInterest(VizInterest),
    Controls(Controls),
    CycleCamera,
}

/// Values subscribed by the visualization software
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct VizInterest {
    pub motors: bool,
    pub servos: bool,
    pub sensors: bool,
    pub state: bool,
}

impl VizInterest {
    pub fn new() -> Self {
        Self {
            motors: false,
            servos: false,
            sensors: false,
            state: false,
        }
    }
}

/// Reexport some types from blimp_onboard_software
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum FlightMode {
    Manual,   // Throttle -> motors speed; Pitch -> motors pitch; Roll -> motors yaw
    Atti,     // Stabilize heading, control thrust vector
    AltiAtti, // Like Atti, but also stabilize altitude
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BlimpState {
    flight_mode: FlightMode,
    altitude: f64,
    desired_altitude: Option<f64>,
    heading: f64,
    desired_heading: Option<f64>,
    pitch: f64,
    roll: f64,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Controls {
    pub throttle_main: f32,       // Generally influences speed
    pub throttle_split: [f32; 4], // Allows you to steer motors individually
    pub sideways: f32,            // Left/right linear movement
    pub elevation: f32,           // Up-down motion
    pub pitch: f32,               // Rotate forward-backward
    pub roll: f32,                // Roll left/right
    pub yaw: f32,                 // Rotate left/right - change heading
    pub desired_flight_mode: FlightMode,
    pub motors_toggles: [bool; 4],
    pub motors_reverse: [bool; 4],
    pub nav_lights: bool,
}
