use super::events::{Code, Key};
use super::util::maybe_utf8;
use hex;
use serde_json::Value;
use std::error::Error;
use std::fmt;

#[derive(PartialEq)]
pub enum APIEvent {
    // from application to IO glue to WormholeCore
    Start,
    AllocateCode(usize), // num_words
    InputCode,
    InputHelperRefreshNameplates,
    InputHelperChooseNameplate(String),
    InputHelperChooseWords(String),
    SetCode(Code),
    Close,
    Send(Vec<u8>),
}

impl fmt::Debug for APIEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::APIEvent::*;
        let t = match *self {
            Start => "Start".to_string(),
            AllocateCode(ref num_words) => {
                format!("AllocateCode({})", num_words)
            }
            InputCode => "InputCode".to_string(),
            InputHelperRefreshNameplates => {
                "InputHelperRefreshNameplates".to_string()
            }
            InputHelperChooseNameplate(ref nameplate) => {
                format!("InputHelperChooseNameplate({})", nameplate)
            }
            InputHelperChooseWords(ref words) => {
                format!("InputHelperChooseWords({})", words)
            }
            SetCode(ref code) => format!("SetCode({:?})", code),
            Close => "Close".to_string(),
            Send(ref msg) => format!("Send({})", maybe_utf8(msg)),
        };
        write!(f, "APIEvent::{}", t)
    }
}

#[derive(Debug, PartialEq)]
pub enum InputHelperError {
    Inactive,
    MustChooseNameplateFirst,
    AlreadyChoseNameplate,
    AlreadyChoseWords,
}

impl fmt::Display for InputHelperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            InputHelperError::Inactive => write!(f, "Inactive"),
            InputHelperError::MustChooseNameplateFirst => {
                write!(f, "Should Choose Nameplate first")
            }
            InputHelperError::AlreadyChoseNameplate => {
                write!(f, "nameplate already chosen, can't go back")
            }
            InputHelperError::AlreadyChoseWords => {
                write!(f, "Words are already chosen")
            }
        }
    }
}

impl Error for InputHelperError {
    fn description(&self) -> &str {
        match *self {
            InputHelperError::Inactive => "Input is not yet started",
            InputHelperError::MustChooseNameplateFirst => {
                "You should input name plate first!"
            }
            InputHelperError::AlreadyChoseNameplate => {
                "Nameplate is already chosen, you can't go back!"
            }
            InputHelperError::AlreadyChoseWords => {
                "Words are already chosen you can't go back!"
            }
        }
    }
}

#[derive(Debug)]
pub enum WormholeError {
    ConnectionError(String),
    ServerError(String),
}

impl Clone for WormholeError {
    fn clone(&self) -> Self {
        use WormholeError::*;
        match self {
            ConnectionError(msg) => ConnectionError(msg.clone()),
            ServerError(msg) => ServerError(msg.clone()),
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum Mood {
    Happy,
    Lonely,
    Errory(WormholeError),
    Scared,
    Unwelcome,
}

impl Mood {
    fn to_protocol_string(self) -> String {
        // this is used for protocol messages as well as debug output
        match self {
            Mood::Happy => "happy".to_string(),
            Mood::Lonely => "lonely".to_string(),
            Mood::Error => "errory".to_string(),
            Mood::Scared => "scary".to_string(),
            Mood::Unwelcome => "unwelcome".to_string(),
        }
    }
}

impl fmt::Display for Mood {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_protocol_string())
    }
}

#[derive(PartialEq)]
pub enum APIAction {
    // from WormholeCore out through IO glue to application
    GotWelcome(Value),
    GotCode(Code), // must be easy to canonically encode into UTF-8 bytes
    GotUnverifiedKey(Key),
    GotVerifier(Vec<u8>),
    GotVersions(Value),
    GotMessage(Vec<u8>),
    GotClosed(Result<Mood, WormholeError>),
}

impl fmt::Debug for APIAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::APIAction::*;
        let t = match *self {
            GotWelcome(ref welcome) => format!("GotWelcome({:?})", welcome),
            GotCode(ref code) => format!("GotCode({:?})", code),
            GotUnverifiedKey(ref _key) => {
                "GotUnverifiedKey(REDACTED)".to_string()
            }
            GotVerifier(ref v) => format!("GotVerifier({})", hex::encode(v)),
            GotVersions(ref versions) => format!("GotVersions({:?})", versions),
            GotMessage(ref msg) => format!("GotMessage({})", maybe_utf8(msg)),
            GotClosed(ref mood) => format!("GotClosed({:?})", mood),
        };
        write!(f, "APIAction::{}", t)
    }
}

// This Private structure prevents external code from forging TimerHandle and
// WSHandle objects (by creating new ones), and the fact that 'id' is not
// public means they can't modify existing ones.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct Private {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TimerHandle {
    id: u32,
    private: Private,
}
impl TimerHandle {
    pub(crate) fn new(id: u32) -> TimerHandle {
        TimerHandle {
            id,
            private: Private {},
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WSHandle {
    id: u32,
    private: Private,
}
impl WSHandle {
    pub(crate) fn new(id: u32) -> WSHandle {
        WSHandle {
            id,
            private: Private {},
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum IOEvent {
    // from IO glue layer into WormholeCore
    TimerExpired(TimerHandle),
    WebSocketConnectionMade(WSHandle),
    WebSocketMessageReceived(WSHandle, String),
    WebSocketConnectionLost(WSHandle, String), // error description
}

#[derive(Debug, PartialEq)]
pub enum IOAction {
    // commands from WormholeCore out to IO glue layer
    StartTimer(TimerHandle, f32),
    CancelTimer(TimerHandle),

    WebSocketOpen(WSHandle, String), // url
    WebSocketSendMessage(WSHandle, String),
    WebSocketClose(WSHandle),
}

// disabled: for now, the glue should call separate do_api/do_io methods
// with an APIEvent or IOEvent respectively
//pub enum InboundEvent { // from IO glue layer
//    IO(IOEvent),
//    API(APIEvent),
//}

#[derive(Debug, PartialEq)]
pub enum Action {
    // to IO glue layer
    // outbound
    IO(IOAction),
    API(APIAction),
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(test)]
mod test {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn test_display() {
        // verify that APIActions have their key redacted
        let w: Value = json!("howdy");
        assert_eq!(
            format!("{:?}", APIAction::GotWelcome(w)),
            r#"APIAction::GotWelcome(String("howdy"))"#
        );
        assert_eq!(
            format!("{:?}", APIAction::GotCode(Code("4-code".into()))),
            r#"APIAction::GotCode(Code("4-code"))"#
        );
        assert_eq!(
            format!(
                "{:?}",
                APIAction::GotUnverifiedKey(Key("secret_key".into()))
            ),
            r#"APIAction::GotUnverifiedKey(REDACTED)"#
        );
        assert_eq!(
            format!("{:?}", APIAction::GotVerifier("verf".into())),
            r#"APIAction::GotVerifier(76657266)"#
        );
        let v: Value = json!("v1");
        assert_eq!(
            format!("{:?}", APIAction::GotVersions(v)),
            r#"APIAction::GotVersions(String("v1"))"#
        );
        assert_eq!(
            format!("{:?}", APIAction::GotMessage("howdy".into())),
            r#"APIAction::GotMessage((s=howdy))"#
        );
        assert_eq!(
            format!("{:?}", APIAction::GotClosed(Mood::Happy)),
            r#"APIAction::GotClosed(Happy)"#
        );
    }
}
