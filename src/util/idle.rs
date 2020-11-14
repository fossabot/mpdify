use crate::mpd_protocol::IdleSubsystem;
use log::debug;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::broadcast;

#[derive(Debug, Copy, Clone)]
pub struct IdleMessage {
    pub what: IdleSubsystem,
    pub when: Instant,
}

pub struct IdleBus {
    channel: broadcast::Sender<IdleMessage>,
}

impl IdleBus {
    #[must_use]
    pub fn new() -> Arc<IdleBus> {
        let (channel, _) = broadcast::channel(16);
        Arc::new(IdleBus { channel })
    }

    /// Returns a channel for notifications, that can be safely dropped
    pub fn subscribe(&self) -> broadcast::Receiver<IdleMessage> {
        self.channel.subscribe()
    }

    /// Send a notification with the current timestamp,
    /// ignores channel errors caused by no subscriber
    pub fn notify(&self, system: IdleSubsystem) {
        debug!["Notifying change in {:?}", system];
        let _ = self.channel.send(IdleMessage {
            what: system,
            when: Instant::now(),
        });
    }
}
