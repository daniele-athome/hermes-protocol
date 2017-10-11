extern crate base64;
#[macro_use]
extern crate error_chain;
extern crate snips_queries_ontology;
#[cfg(feature = "ffi")]
extern crate libc;
#[cfg(any(feature = "mqtt", feature = "inprocess"))]
#[macro_use]
extern crate log;
#[cfg(feature = "mqtt")]
extern crate rumqtt;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[cfg(feature = "mqtt")]
extern crate strum;
#[cfg(feature = "mqtt")]
#[macro_use]
extern crate strum_macros;

mod errors;
#[cfg(feature = "mqtt")]
mod mqtt;
#[cfg(feature = "inprocess")]
mod inprocess;
#[cfg(feature = "ffi")]
pub mod ffi;

pub use errors::*;
#[cfg(feature = "mqtt")]
pub use mqtt::MqttHermesProtocolHandler;
#[cfg(feature = "inprocess")]
pub use inprocess::InProcessHermesProtocolHandler;

use snips_queries_ontology::{IntentClassifierResult, Slot};

/// A struct wrapping a callback with one argument, create one with the `new` method
pub struct Callback<T> {
    callback: Box<Fn(&T) -> () + Send + Sync>
}

impl<T> Callback<T> {
    pub fn new<F: 'static>(handler: F) -> Callback<T> where F: Fn(&T) -> () + Send + Sync {
        Callback { callback: Box::new(handler) }
    }

    pub fn call(&self, arg: &T) { (self.callback)(arg) }
}

/// A struct wrapping a callback with no argument, create one with the `new` method
pub struct Callback0 {
    callback: Box<Fn() -> () + Send + Sync>
}

impl Callback0 {
    pub fn new<F: 'static>(handler: F) -> Callback0 where F: Fn() -> () + Send + Sync {
        Callback0 { callback: Box::new(handler) }
    }

    pub fn call(&self) { (self.callback)() }
}

/// A facade to interact with a component that can be toggled on an off at a specific site
pub trait ToggleableFacade: Send + Sync {
    fn publish_toggle_on(&self, site: SiteMessage) -> Result<()>;
    fn publish_toggle_off(&self, site: SiteMessage) -> Result<()>;
}

/// The facade a component that can be toggled on an off at a specific site must use to receive
/// its orders
pub trait ToggleableBackendFacade: Send + Sync {
    fn subscribe_toggle_on(&self, handler: Callback<SiteMessage>) -> Result<()>;
    fn subscribe_toggle_off(&self, handler: Callback<SiteMessage>) -> Result<()>;
}

/// The facade to interact with the hotword component
pub trait HotwordFacade: ComponentFacade + ToggleableFacade {
    fn subscribe_detected(&self, handler: Callback<SiteMessage>) -> Result<()>;
}

/// The facade the hotword feature must use receive its orders and publish detected hotwords
pub trait HotwordBackendFacade: ComponentBackendFacade + ToggleableBackendFacade {
    fn publish_detected(&self, site: SiteMessage) -> Result<()>;
}

/// The facade used to toggle on and of the sound feedback at a specific site
pub trait SoundFeedbackFacade: ToggleableFacade {}

/// The facade a component that manages sound feedback must use to receive its orders
pub trait SoundFeedbackBackendFacade: ToggleableBackendFacade {}

/// The facade to interact with the automatic speech recognition component
pub trait AsrFacade: ComponentFacade + ToggleableFacade {
    fn subscribe_text_captured(&self, handler: Callback<TextCapturedMessage>) -> Result<()>;
    fn subscribe_partial_text_captured(&self, handler: Callback<TextCapturedMessage>) -> Result<()>;
}

/// The facade the automatic speech recognition must use to receive its orders and publish
/// recognized text
pub trait AsrBackendFacade: ComponentBackendFacade + ToggleableBackendFacade {
    fn publish_text_captured(&self, text_captured: TextCapturedMessage) -> Result<()>;
    fn publish_partial_text_captured(&self, text_captured: TextCapturedMessage) -> Result<()>;
}

/// The facade to interact with the text to speech component
pub trait TtsFacade: ComponentFacade {
    fn publish_say(&self, to_say: SayMessage) -> Result<()>;
    fn subscribe_say_finished(&self, handler: Callback<SayFinishedMessage>) -> Result<()>;
}

/// The facade the text to speech must use to receive its orders and advertise when it has finished
pub trait TtsBackendFacade: ComponentBackendFacade {
    fn publish_say_finished(&self, status: SayFinishedMessage) -> Result<()>;
    fn subscribe_say(&self, handler: Callback<SayMessage>) -> Result<()>;
}

/// The facade to interact with the natural language understanding component
pub trait NluFacade: ComponentFacade {
    fn publish_query(&self, query: NluQueryMessage) -> Result<()>;
    fn publish_partial_query(&self, query: NluSlotQueryMessage) -> Result<()>;
    fn subscribe_slot_parsed(&self, handler: Callback<NluSlotMessage>) -> Result<()>;
    fn subscribe_intent_parsed(&self, handler: Callback<NluIntentMessage>) -> Result<()>;
    fn subscribe_intent_not_recognized(&self, handler: Callback<NluIntentNotRecognizedMessage>) -> Result<()>;
}

/// The facade the natural language understanding must use to receive its orders and publish
/// its results
pub trait NluBackendFacade: ComponentBackendFacade {
    fn subscribe_query(&self, handler: Callback<NluQueryMessage>) -> Result<()>;
    fn subscribe_partial_query(&self, handler: Callback<NluSlotQueryMessage>) -> Result<()>;
    fn publish_slot_parsed(&self, slot: NluSlotMessage) -> Result<()>;
    fn publish_intent_parsed(&self, intent: NluIntentMessage) -> Result<()>;
    fn publish_intent_not_recognized(&self, status: NluIntentNotRecognizedMessage) -> Result<()>;
}

/// The facade to interact with the audio server
pub trait AudioServerFacade: ComponentFacade {
    fn publish_play_bytes(&self, bytes: PlayBytesMessage) -> Result<()>;
    fn subscribe_play_finished(&self, handler: Callback<PlayFinishedMessage>) -> Result<()>;
    fn subscribe_audio_frame(&self, site_id: SiteId, handler: Callback<AudioFrameMessage>) -> Result<()>;
}

/// The facade the audio server must use to receive its orders and advertise when it has finished
pub trait AudioServerBackendFacade: ComponentBackendFacade {
    fn subscribe_play_bytes(&self, site_id: SiteId, handler: Callback<PlayBytesMessage>) -> Result<()>;
    fn publish_play_finished(&self, status: PlayFinishedMessage) -> Result<()>;
    fn publish_audio_frame(&self, frame: AudioFrameMessage) -> Result<()>;
}

/// A generic facade used to interact with a component
pub trait ComponentFacade: Send + Sync {
    fn publish_version_request(&self) -> Result<()>;
    fn subscribe_version(&self, handler: Callback<VersionMessage>) -> Result<()>;
    fn subscribe_error(&self, handler: Callback<ErrorMessage>) -> Result<()>;
}

/// A generic facade all components must use to publish their errors and versions (when requested)
pub trait ComponentBackendFacade: Send + Sync {
    fn subscribe_version_request(&self, handler: Callback0) -> Result<()>;
    fn publish_version(&self, version: VersionMessage) -> Result<()>;
    fn publish_error(&self, error: ErrorMessage) -> Result<()>;
}

/// The facade to use to interact with the dialogue manager, this is the principal interface that a
/// lambda should use
pub trait DialogueFacade: ComponentFacade + ToggleableFacade {
    fn subscribe_session_started(&self, handler: Callback<SessionStartedMessage>) -> Result<()>;
    fn subscribe_intent(&self, intent_name: String, handler: Callback<IntentMessage>) -> Result<()>;
    fn subscribe_intents(&self, handler: Callback<IntentMessage>) -> Result<()>;
    fn subscribe_session_ended(&self, handler: Callback<SessionEndedMessage>) -> Result<()>;
    fn publish_start_session(&self, start_session: StartSessionMessage) -> Result<()>;
    fn publish_continue_session(&self, continue_session: ContinueSessionMessage) -> Result<()>;
    fn publish_end_session(&self, end_session: EndSessionMessage) -> Result<()>;
}

/// The facade the dialogue manager must use to interact with the lambdas
pub trait DialogueBackendFacade: ComponentBackendFacade + ToggleableBackendFacade {
    fn publish_session_started(&self, status: SessionStartedMessage) -> Result<()>;
    fn publish_intent(&self, intent: IntentMessage) -> Result<()>;
    fn publish_session_ended(&self, status: SessionEndedMessage) -> Result<()>;
    fn subscribe_start_session(&self, handler: Callback<StartSessionMessage>) -> Result<()>;
    fn subscribe_continue_session(&self, handler: Callback<ContinueSessionMessage>) -> Result<()>;
    fn subscribe_end_session(&self, handler: Callback<EndSessionMessage>) -> Result<()>;
}

pub trait HermesProtocolHandler: Send + Sync {
    fn hotword(&self) -> Box<HotwordFacade>;
    fn sound_feedback(&self) -> Box<SoundFeedbackFacade>;
    fn asr(&self) -> Box<AsrFacade>;
    fn tts(&self) -> Box<TtsFacade>;
    fn nlu(&self) -> Box<NluFacade>;
    fn audio_server(&self) -> Box<AudioServerFacade>;
    fn dialogue(&self) -> Box<DialogueFacade>;
    fn hotword_backend(&self) -> Box<HotwordBackendFacade>;
    fn sound_feedback_backend(&self) -> Box<SoundFeedbackBackendFacade>;
    fn asr_backend(&self) -> Box<AsrBackendFacade>;
    fn tts_backend(&self) -> Box<TtsBackendFacade>;
    fn nlu_backend(&self) -> Box<NluBackendFacade>;
    fn audio_server_backend(&self) -> Box<AudioServerBackendFacade>;
    fn dialogue_backend(&self) -> Box<DialogueBackendFacade>;
}

pub trait HermesMessage: ::std::fmt::Debug {}

pub type SiteId = String;

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SiteMessage {
    /// The site concerned
    #[serde(rename = "siteId")]
    pub site_id: SiteId,
}

impl HermesMessage for SiteMessage {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct TextCapturedMessage {
    /// The text captured
    pub text: String,
    /// The likelihood of the capture
    pub likelihood: f32,
    /// The duration it took to do the processing
    pub seconds: f32,
    /// The site where the text was captured
    #[serde(rename = "siteId")]
    pub site_id: SiteId,
}

impl HermesMessage for TextCapturedMessage {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct NluQueryMessage {
    /// The text to run the NLU on
    pub text: String,
    /// An optional list of intents to restrict the NLU resolution on
    #[serde(rename = "intentFilter")]
    pub intent_filter: Option<Vec<String>>,
    /// An optional id for the request, if provided it will be passed back in the
    /// response `NluIntentMessage` or `NluIntentNotRecognizedMessage`
    pub id: Option<String>
}

impl HermesMessage for NluQueryMessage {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct NluSlotQueryMessage {
    /// The text to run the slot detection on
    pub text: String,
    #[serde(rename = "intentName")]
    /// The intent to use when doing the slot detection
    pub intent_name: String,
    /// The slot to search
    #[serde(rename = "slotName")]
    pub slot_name: String,
    /// An optional id for the request, if provided it will be passed back in the
    /// response `SlotMessage`
    pub id: Option<String>,
}

impl HermesMessage for NluSlotQueryMessage {}


#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct PlayBytesMessage {
    /// An id for the request, it will be passed back in the `PlayFinishedMessage`
    pub id: String,
    /// The bytes of the wav to play (should be a regular wav with header)
    /// Note that serde json serialization is provided but in practice most handler impl will want
    /// to avoid the base64 encoding/decoding and give this a special treatment
    #[serde(rename = "wavBytes", serialize_with = "as_base64", deserialize_with = "from_base64")]
    pub wav_bytes: Vec<u8>,
    /// The site where the bytes should be played
    #[serde(rename = "siteId")]
    pub site_id: SiteId,
}

impl HermesMessage for PlayBytesMessage {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct AudioFrameMessage {
    /// The bytes of the wav frame (should be a regular wav with header)
    /// Note that serde json serialization is provided but in practice most handler impl will want
    /// to avoid the base64 encoding/decoding and give this a special treatment
    #[serde(rename = "wavFrame", serialize_with = "as_base64", deserialize_with = "from_base64")]
    pub wav_frame: Vec<u8>,
    /// The site this frame originates from
    #[serde(rename = "siteId")]
    pub site_id: SiteId,
}

impl HermesMessage for AudioFrameMessage {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct PlayFinishedMessage {
    /// The id of the `PlayBytesMessage` which bytes finished playing
    pub id: String
}

impl HermesMessage for PlayFinishedMessage {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SayMessage {
    /// The text to say
    pub text: String,
    /// The lang to use when saying the `text`, will use en_GB if not provided
    pub lang: Option<String>,
    /// An optional id for the request, it will be passed back in the `SayFinishedMessage`
    pub id: Option<String>,
    /// The site where the message should be said
    #[serde(rename = "siteId")]
    pub site_id: SiteId,
}

impl HermesMessage for SayMessage {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SayFinishedMessage {
    /// The id of the `SayMessage` which was has been said
    pub id: Option<String>
}

impl HermesMessage for SayFinishedMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NluSlotMessage {
    /// The id of the `NluSlotQueryMessage` that was processed
    pub id: Option<String>,
    /// The input that was processed
    pub input: String,
    /// The intent used to find the slot
    pub intent: String,
    /// The resulting slot, if found
    pub slot: Option<Slot>,
}

impl HermesMessage for NluSlotMessage {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct NluIntentNotRecognizedMessage {
    /// The id of the `NluQueryMessage` that was processed
    pub id: Option<String>,
    /// The text that didn't match any intent
    pub input: String,
}

impl HermesMessage for NluIntentNotRecognizedMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NluIntentMessage {
    /// The id of the `NluQueryMessage` that was processed
    pub id: Option<String>,
    /// The input that was processed
    pub input: String,
    /// The result of the intent classification
    pub intent: IntentClassifierResult,
    /// The detected slots, if any
    pub slots: Option<Vec<Slot>>,
}

impl HermesMessage for NluIntentMessage {}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct IntentMessage {
    /// The session in with this intent was detected
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// The custom data that was given at the session creation
    #[serde(rename = "customData")]
    pub custom_data: Option<String>,
    /// The input that generated this intent
    pub input: String,
    /// The result of the intent classification
    pub intent: IntentClassifierResult,
    /// The detected slots, if any
    pub slots: Option<Vec<Slot>>,
}

impl HermesMessage for IntentMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "from")]
pub enum SessionInit {
    /// Interaction was initiated by the user
    User,
    /// Interaction was initiated by a lambda
    Lambda { action: SessionAction }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SessionAction {
    /// The text to say to the user
    pub text: String,
    /// Whether of not the lambda expects the user to respond, if set to `false` the session will
    /// be closed once the text has been said
    #[serde(rename = "expectResponse")]
    pub expect_response: bool,
    /// An optional list of intent name to restrict the parsing of the user response to. This will
    /// be ignored if `expect_response` is set to `false`
    #[serde(rename = "intentFilter")]
    pub intent_filter: Option<Vec<String>>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StartSessionMessage {
    /// The way this session was created
    pub init: SessionInit,
    /// The custom data that was given at the session creation
    #[serde(rename = "customData")]
    pub custom_data: Option<String>,
    /// The site where the session should be started, a value of `None` will be interpreted as the
    /// default one
    #[serde(rename = "siteId")]
    pub site_id: Option<SiteId>,
}

impl HermesMessage for StartSessionMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SessionStartedMessage {
    /// The id of the session that was started
    pub session_id: String,
    /// An optional piece of data that will be given back in `IntentMessage` and
    /// `SessionAbortedMessage` that are related to this session
    #[serde(rename = "customData")]
    pub custom_data: Option<String>,
    /// The site on which this session was started
    pub site_id: SiteId
}

impl HermesMessage for SessionStartedMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ContinueSessionMessage {
    /// The id of the session this action applies to
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// The action to perform
    pub action: SessionAction,
}

impl HermesMessage for ContinueSessionMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EndSessionMessage {
    /// The id of the session to end
    #[serde(rename = "sessionId")]
    pub session_id: String,
}

impl HermesMessage for EndSessionMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SessionEndedMessage {
    /// The id of the session that was terminated
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// The custom data that was given at the session creation
    #[serde(rename = "customData")]
    pub custom_data: Option<String>,
    /// Set to true if the session was aborted by the used
    pub aborted: Option<bool>,
}

impl HermesMessage for SessionEndedMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VersionMessage {
    /// The version of the component
    pub version: semver::Version,
}

impl HermesMessage for VersionMessage {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ErrorMessage {
    /// The error that occurred
    pub error: String,
    /// Optional additional information on the context in which the error occurred
    pub context: Option<String>,
}

impl HermesMessage for ErrorMessage {}

fn as_base64<S>(bytes: &[u8], serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: serde::Serializer {
    serializer.serialize_str(&base64::encode(bytes))
}

fn from_base64<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
    where D: serde::Deserializer<'de> {
    use serde::de::Error;
    use serde::Deserialize;
    String::deserialize(deserializer)
        .and_then(|string| base64::decode(&string).map_err(|err| Error::custom(err.to_string())))
}
