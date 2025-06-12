pub const PROTOCOL_ORGANIZATION: &str = "spacecoffee";
pub const PROTOCOL_PROJECT: &str = "blimp";
pub const PROTOCOL_VERSION: u16 = 1;

/// Messages sent by the server
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub enum MessageG2V {
    MotorSpeed { id: u8, speed: f32 },
    ServoPosition { id: u8, angle: f32 },
    SensorData { id: String, data: f64 },
}

/// Messages sent by the client
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub enum MessageV2G {
    DeclareInterest(VizInterest),
    Controls(Controls),
}

/// Values subscribed by the visualization software
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct VizInterest {
    pub motors: bool,
    pub servos: bool,
    pub sensors: bool,
}

impl VizInterest {
    pub fn new() -> Self {
        Self {
            motors: false,
            servos: false,
            sensors: false,
        }
    }
}

/// Reexport Controls from blimp_onboard_software
pub use blimp_onboard_software::obsw_algo::Controls;
pub use blimp_onboard_software::obsw_algo::FlightMode;
