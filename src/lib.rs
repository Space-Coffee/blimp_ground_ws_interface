use serde;

// Ground to visualization
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum MessageG2V {
    MotorSpeed { id: u8, speed: i32 },
    ServoPosition { id: u8, angle: i32 },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VisInterest {
    motors: bool,
    servos: bool,
}

// Visualization to ground
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum MessageV2G {
    DeclareInterest(VisInterest),
}
