use std::fmt;

use semver;

use serde::{Deserialize, Serialize};

pub mod asr;
pub mod audio_server;
pub mod dialogue;
pub mod hotword;
pub mod injection;
pub mod nlu;
pub mod tts;
pub mod vad;

pub use self::asr::*;
pub use self::audio_server::*;
pub use self::dialogue::*;
pub use self::hotword::*;
pub use self::injection::*;
pub use self::nlu::*;
pub use self::tts::*;
pub use self::vad::*;

pub trait HermesMessage<'de>: fmt::Debug + Deserialize<'de> + Serialize {}

#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Debug, Clone)]
pub enum SnipsComponent {
    Asr,
    Nlu,
    Dialogue,
    Hotword,
    Injection,
    Tts,
    AudioServer
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteMessage {
    /// The site concerned
    pub site_id: String,
    /// An optional session id if there is a related session
    pub session_id: Option<String>,
}

impl Default for SiteMessage {
    fn default() -> Self {
        Self {
            site_id: "default".into(),
            session_id: None,
        }
    }
}

impl<'de> HermesMessage<'de> for SiteMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionMessage {
    /// The version of the component
    pub version: semver::Version,
}

impl<'de> HermesMessage<'de> for VersionMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessage {
    /// An optional session id if there is a related session
    pub session_id: Option<String>,
    /// The error that occurred
    pub error: String,
    /// Optional additional information on the context in which the error occurred
    pub context: Option<String>,
}

impl<'de> HermesMessage<'de> for ErrorMessage {}

fn as_base64<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&base64::encode(bytes))
}

fn from_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)
        .and_then(|string| base64::decode(&string).map_err(|err| Error::custom(err.to_string())))
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadMessage {
    /// Optional id associated to a reload operation of a component
    pub component: SnipsComponent,
    pub load_id: Option<String>,
}

impl<'de> HermesMessage<'de> for LoadMessage {}


#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteLoadMessage {
    /// Optional id associated to a reload operation of a component
    pub component: SnipsComponent,
    pub load_id: Option<String>,
    pub site_id: String
}

impl<'de> HermesMessage<'de> for SiteLoadMessage {}
