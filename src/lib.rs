use serde;

// Ground to visualization
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum MessageG2V {
    MotorSpeed { id: u8, speed: i32 },
    ServoPosition { id: u8, angle: i16 },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VizInterest {
    pub motors: bool,
    pub servos: bool,
}

impl VizInterest {
    pub fn new() -> Self {
        Self {
            motors: false,
            servos: false,
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Controls {
    pub throttle: i32,
    pub elevation: i32,
    pub yaw: i32,
}

// Visualization to ground
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum MessageV2G {
    DeclareInterest(VizInterest),
    Controls(Controls),
}
