use serde;

// Ground to visualization
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum MessageG2V {
    MotorSpeed { id: u8, speed: i32 },
    ServoPosition { id: u8, angle: i32 },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VisInterest {
    pub motors: bool,
    pub servos: bool,
}

impl VisInterest {
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
    pub pitch: i32,
    pub roll: i32,
}

// Visualization to ground
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum MessageV2G {
    DeclareInterest(VisInterest),
    Controls(Controls),
}
