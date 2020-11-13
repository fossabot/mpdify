use serde::export::Formatter;
use serde::Serialize;
use std::fmt;

/// Playback status for StatusResponse
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackStatus {
    Play,
    Pause,
    Stop,
}

/// Response for the status command
#[derive(Debug, PartialEq, Serialize)]
pub struct StatusResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<u32>,
    pub state: PlaybackStatus,
}

/// Holder for HandlerOutput::Serialize
pub struct OutputData {
    pub data: Vec<Box<dyn erased_serde::Serialize + Send>>,
}

impl OutputData {
    pub fn from<T: 'static>(value: T) -> OutputData
    where
        T: erased_serde::Serialize + Send,
    {
        let mut data: Vec<Box<dyn erased_serde::Serialize + Send>> = Vec::new();
        data.push(Box::from(value));
        OutputData { data }
    }

    pub fn push<T: 'static>(&mut self, value: T)
    where
        T: erased_serde::Serialize + Send,
    {
        self.data.push(Box::from(value));
    }
}

impl fmt::Debug for OutputData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for item in &self.data {
            serde_fmt::to_debug(item.as_ref()).fmt(f)?;
            f.write_str(", ")?;
        }
        Ok(())
    }
}
