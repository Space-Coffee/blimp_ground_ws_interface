/// Messages sent by the server
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub enum MessageG2V {
    MotorSpeed { id: u8, speed: i32 },
    ServoPosition { id: u8, angle: i16 },
    SensorData { id: String, data: f64 },
}

/// Messages sent by the client
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Copy, Clone)]
pub enum MessageV2G {
    DeclareInterest(VizInterest),
    Controls(Controls),
}

/// Values subscribed by the visualization software
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Copy, Clone)]
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

/// Values for controlling the blimp
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Copy, Clone)]
pub struct Controls {
    pub throttle: i32,
    pub elevation: i32,
    pub yaw: i32,
}
