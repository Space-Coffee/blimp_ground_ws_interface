use crate::{PROTOCOL_ORGANIZATION, PROTOCOL_PROJECT, PROTOCOL_VERSION};
use phf::phf_map;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlimpSubprotocolFlavour {
    Json,
    Postcard,
}

#[derive(Debug, PartialEq)]
pub enum SubprotocolValidationError {
    InvalidProtocol,
    IncompatibleVersion,
    UnsupportedFlavour
}

#[derive(Debug, PartialEq)]
pub struct BlimpSubprotocol {
    pub version: u16,
    pub flavour: BlimpSubprotocolFlavour
}

impl Default for BlimpSubprotocol {
    fn default() -> Self {
        BlimpSubprotocol { version: PROTOCOL_VERSION, flavour: BlimpSubprotocolFlavour::Postcard }
    }
}
pub const SUPPORTED_VERSIONS: [u16; 1] = [1];
pub static SUPPORTED_FLAVOURS: phf::Map<&'static str, BlimpSubprotocolFlavour> = phf_map! {
    "json" => BlimpSubprotocolFlavour::Json,
    "postcard" => BlimpSubprotocolFlavour::Postcard,
};
impl FromStr for BlimpSubprotocol {
    type Err = SubprotocolValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let namespaces: Vec<&str> = s.split(".").collect();
        if namespaces.len() != 4 {
            return Err(SubprotocolValidationError::InvalidProtocol)
        }
        let [organization, project, version, flavour] = namespaces.as_slice() else {
            return Err(SubprotocolValidationError::InvalidProtocol)
        };
        if *organization != PROTOCOL_ORGANIZATION || *project != PROTOCOL_PROJECT {
            return Err(SubprotocolValidationError::InvalidProtocol)
        }
        let Some(parsed_version) = version.strip_prefix("v").and_then(|v| v.parse::<u16>().ok()) else {
            return Err(SubprotocolValidationError::InvalidProtocol)
        };
        if !SUPPORTED_VERSIONS.contains(&parsed_version) {
            return Err(SubprotocolValidationError::IncompatibleVersion)
        }
        let Some(parsed_flavour) = SUPPORTED_FLAVOURS.get(flavour) else {
            return Err(SubprotocolValidationError::UnsupportedFlavour)
        };
        Ok(Self {version: parsed_version, flavour: parsed_flavour.to_owned()})
    }
}

impl Display for BlimpSubprotocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flavour_name = match self.flavour {
            BlimpSubprotocolFlavour::Json => "json",
            BlimpSubprotocolFlavour::Postcard => "postcard"
        };

        write!(f, "{}", format!("{}.{}.v{}.{}", PROTOCOL_ORGANIZATION, PROTOCOL_PROJECT, self.version, flavour_name))
    }
}