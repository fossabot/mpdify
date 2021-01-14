use crate::handlers::aspotify::context::PlayContext;
use crate::handlers::aspotify::playback::CachedPlayback;
use crate::mpd_protocol::{
    HandlerOutput, HandlerResult, OutputData, OutputsResponse, PlaybackStatus, StatusDurations,
    StatusPlaylistInfo, StatusResponse,
};
use aspotify::{CurrentPlayback, Device, PlayingType, RepeatState};
use std::sync::Arc;
use std::time::Duration;

pub fn build_outputs_result(devices: Vec<Device>) -> HandlerResult {
    let mut out = OutputData::empty();
    for (pos, device) in devices.iter().enumerate() {
        out.push(OutputsResponse {
            outputid: pos,
            outputname: device.name.clone(),
            outputenabled: device.is_active,
            plugin: "spotify".to_string(),
        })
    }
    Ok(HandlerOutput::Data(out))
}

pub fn build_status_result(input: Arc<CachedPlayback>, context: Arc<PlayContext>) -> HandlerResult {
    match &input.data {
        None => Ok(HandlerOutput::from(StatusResponse {
            volume: None,
            state: PlaybackStatus::Stop,
            random: false,
            repeat: false,
            single: false,
            durations: None,
            playlist_info: None,
        })),
        Some(data) => {
            let spotify_id = data
                .currently_playing
                .item
                .as_ref()
                .map(extract_id)
                .flatten()
                .unwrap_or_else(|| String::from("unknown"));
            let pos = context.position_for_id(spotify_id.as_str());
            Ok(HandlerOutput::from(StatusResponse {
                volume: data.device.volume_percent,
                state: if data.currently_playing.is_playing {
                    PlaybackStatus::Play
                } else {
                    PlaybackStatus::Pause
                },
                random: data.shuffle_state,
                repeat: RepeatState::Off.ne(&data.repeat_state),
                single: RepeatState::Track.eq(&data.repeat_state),
                durations: extract_durations(&data, input.get_elapsed()),
                playlist_info: Some(StatusPlaylistInfo::new(context.size(), pos)),
            }))
        }
    }
}

pub fn extract_durations(
    data: &CurrentPlayback,
    elapsed: Option<Duration>,
) -> Option<StatusDurations> {
    let duration = data.currently_playing.item.as_ref().map(|item| match item {
        PlayingType::Track(track) => track.duration,
        PlayingType::Episode(ep) => ep.duration,
        PlayingType::Ad(ad) => ad.duration,
        PlayingType::Unknown(u) => u.duration,
    });
    if let Some(elapsed) = elapsed {
        if let Some(duration) = duration {
            return Some(StatusDurations { elapsed, duration });
        }
    }
    None
}

pub fn extract_id(item: &PlayingType) -> Option<String> {
    match item {
        PlayingType::Track(track) => track.id.clone(),
        PlayingType::Episode(ep) => Some(ep.id.clone()),
        PlayingType::Ad(track) => track.id.clone(),
        PlayingType::Unknown(track) => track.id.clone(),
    }
}
