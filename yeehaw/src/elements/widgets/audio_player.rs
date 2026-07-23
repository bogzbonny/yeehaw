use {
    std::path::PathBuf,
    std::rc::Rc,
    std::cell::RefCell,
    std::sync::Arc,
    yeehaw_derive::{impl_pane_basics_from, impl_element_from},
    crate::dyn_value::DynVal,
    crate::dyn_location::{DynLocation, DynLocationSet, Size, ZIndex},
    crate::element::{Element, DrawUpdate, Parent},
    crate::draw_region::DrawRegion,
    crate::sorting_hat::ElementID,
    crate::event::{Event, EventResponses, ReceivableEvents, ReceivableEvent},
    crate::ch::{DrawCh, DrawChs2D},
    crate::color::Color,
    crate::style::Style,
    crate::context::Context,
    crate::elements::panes::ParentPaneOfSelectable,
    crate::elements::widgets::button::{Button, ButtonMicroShadow},
    crate::elements::widgets::slider::Slider,
    crate::{Ref, RefMut},
    cpal::traits::{DeviceTrait, HostTrait, StreamTrait},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AudioPlayerState {
    Playing,
    Paused,
    Stopped,
}

/// Shared audio state accessible from both the UI thread and the cpal callback thread.
struct AudioContext {
    samples: Vec<f32>,
    sample_rate: u32,
    position: usize,
    state: AudioPlayerState,
}

impl AudioContext {
    fn fraction(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.position as f64 / self.samples.len() as f64
    }
}

/// AudioPlayer element that plays audio files using cpal.
///
/// Layout:
/// - Row 0: Slider showing playback position (draggable for seeking)
/// - Row 1: Prev, Play/Pause, Stop, Next buttons with MicroShadow styling
#[derive(Clone)]
pub struct AudioPlayer {
    pub pane: ParentPaneOfSelectable,

    // Audio sources
    sources: Rc<RefCell<Vec<PathBuf>>>,
    current_index: Rc<RefCell<usize>>,

    // Shared audio state (cross-thread with cpal callback)
    audio_ctx: Rc<RefCell<Option<Arc<parking_lot::Mutex<AudioContext>>>>>,

    // Playback stream handle
    stream: Rc<RefCell<Option<cpal::Stream>>>,

    // UI elements
    slider: Rc<RefCell<Slider>>,
    play_pause_btn: Rc<RefCell<Button>>,
    stop_btn: Rc<RefCell<Button>>,
    prev_btn: Rc<RefCell<Button>>,
    next_btn: Rc<RefCell<Button>>,
}

impl AudioPlayer {
    /// Create a new AudioPlayer with the given audio file paths.
    pub fn new(ctx: &Context, sources: Vec<PathBuf>) -> Self {
        let pane = ParentPaneOfSelectable::new(ctx);
        pane.set_dyn_width(DynVal::FULL);
        pane.set_dyn_height(DynVal::new_fixed(2));

        // Create slider
        let slider = Slider::new_basic_line(ctx);
        slider.pane.set_dyn_width(DynVal::FULL);
        slider.pane.set_dyn_height(DynVal::new_fixed(1));
        slider.pane.set_at(DynVal::new_fixed(0), DynVal::new_fixed(0));
        slider.set_position(0.0);

        // Create buttons with micro shadow
        let shadow = ButtonMicroShadow::default();
        let prev_btn = Button::new(ctx, "⏮ ").with_micro_shadow(ctx, shadow.clone());
        let play_pause_btn = Button::new(ctx, "▶").with_micro_shadow(ctx, shadow.clone());
        let stop_btn = Button::new(ctx, "⏹").with_micro_shadow(ctx, shadow.clone());
        let next_btn = Button::new(ctx, "⏭ ").with_micro_shadow(ctx, shadow.clone());

        let sources_rc = Rc::new(RefCell::new(sources));
        let current_index = Rc::new(RefCell::new(0usize));
        let audio_ctx: Rc<RefCell<Option<Arc<parking_lot::Mutex<AudioContext>>>>> = Rc::new(RefCell::new(None));
        let stream: Rc<RefCell<Option<cpal::Stream>>> = Rc::new(RefCell::new(None));

        // Wire up slider callback for seeking
        {
            let audio_ctx_clone = audio_ctx.clone();
            slider.set_fn(Box::new(move |_: Context, sl: &Slider| {
                let pos = sl.get_position();
                if let Some(ctx) = audio_ctx_clone.borrow().as_ref() {
                    let mut audio = ctx.lock();
                    let new_pos = (pos * audio.samples.len() as f64) as usize;
                    audio.position = new_pos.min(audio.samples.len());
                }
                EventResponses::default()
            }));
        }

        // Wire up Play/Pause button
        {
            let audio_ctx_clone = audio_ctx.clone();
            let stream_clone = stream.clone();
            let play_pause_btn_clone = Rc::new(RefCell::new(play_pause_btn.clone()));

            play_pause_btn.set_fn(Box::new(move |_, ctx| {
                if let Some(ac) = audio_ctx_clone.borrow().as_ref() {
                    let mut audio = ac.lock();
                    match audio.state {
                        AudioPlayerState::Playing => {
                            audio.state = AudioPlayerState::Paused;
                            if let Some(s) = stream_clone.borrow().as_ref() {
                                let _ = s.pause();
                            }
                            *play_pause_btn_clone.borrow_mut().text.borrow_mut() = "▶".to_string();
                            Self::refresh_button(&play_pause_btn_clone, &ctx);
                        }
                        AudioPlayerState::Paused => {
                            audio.state = AudioPlayerState::Playing;
                            if let Some(s) = stream_clone.borrow().as_ref() {
                                let _ = s.play();
                            }
                            *play_pause_btn_clone.borrow_mut().text.borrow_mut() = "⏸".to_string();
                            Self::refresh_button(&play_pause_btn_clone, &ctx);
                        }
                        AudioPlayerState::Stopped => {
                            audio.state = AudioPlayerState::Playing;
                            audio.position = 0;
                            if let Some(s) = stream_clone.borrow().as_ref() {
                                let _ = s.play();
                            }
                            *play_pause_btn_clone.borrow_mut().text.borrow_mut() = "⏸".to_string();
                            Self::refresh_button(&play_pause_btn_clone, &ctx);
                        }
                    }
                }
                EventResponses::default()
            }));
        }

        // Wire up Stop button
        {
            let audio_ctx_clone = audio_ctx.clone();
            let stream_clone = stream.clone();
            let play_pause_btn_clone = Rc::new(RefCell::new(play_pause_btn.clone()));

            stop_btn.set_fn(Box::new(move |_, ctx| {
                if let Some(ac) = audio_ctx_clone.borrow().as_ref() {
                    let mut audio = ac.lock();
                    audio.state = AudioPlayerState::Stopped;
                    audio.position = 0;
                }
                if let Some(s) = stream_clone.borrow().as_ref() {
                    let _ = s.pause();
                }
                *play_pause_btn_clone.borrow_mut().text.borrow_mut() = "▶".to_string();
                Self::refresh_button(&play_pause_btn_clone, &ctx);
                EventResponses::default()
            }));
        }

        // Wire up Prev button
        {
            let sources_clone = sources_rc.clone();
            let current_index_clone = current_index.clone();
            let audio_ctx_clone = audio_ctx.clone();
            let stream_clone = stream.clone();
            let play_pause_btn_clone = Rc::new(RefCell::new(play_pause_btn.clone()));
            let slider_clone = Rc::new(RefCell::new(slider.clone()));

            prev_btn.set_fn(Box::new(move |_, _| {
                let mut idx = *current_index_clone.borrow();
                let sources = sources_clone.borrow();
                if sources.is_empty() {
                    return EventResponses::default();
                }
                if idx > 0 {
                    idx -= 1;
                } else {
                    idx = sources.len() - 1;
                }
                *current_index_clone.borrow_mut() = idx;
                AudioPlayer::load_track(
                    &sources[idx],
                    &audio_ctx_clone,
                    &stream_clone,
                    &play_pause_btn_clone,
                    &slider_clone,
                );
                EventResponses::default()
            }));
        }

        // Wire up Next button
        {
            let sources_clone = sources_rc.clone();
            let current_index_clone = current_index.clone();
            let audio_ctx_clone = audio_ctx.clone();
            let stream_clone = stream.clone();
            let play_pause_btn_clone = Rc::new(RefCell::new(play_pause_btn.clone()));
            let slider_clone = Rc::new(RefCell::new(slider.clone()));

            next_btn.set_fn(Box::new(move |_, _| {
                let mut idx = *current_index_clone.borrow();
                let sources = sources_clone.borrow();
                if sources.is_empty() {
                    return EventResponses::default();
                }
                idx = (idx + 1) % sources.len();
                *current_index_clone.borrow_mut() = idx;
                AudioPlayer::load_track(
                    &sources[idx],
                    &audio_ctx_clone,
                    &stream_clone,
                    &play_pause_btn_clone,
                    &slider_clone,
                );
                EventResponses::default()
            }));
        }

        // Position buttons in row 1
        prev_btn.pane.set_at(DynVal::new_fixed(0), DynVal::new_fixed(1));
        play_pause_btn.pane.set_at(DynVal::new_fixed(4), DynVal::new_fixed(1));
        stop_btn.pane.set_at(DynVal::new_fixed(8), DynVal::new_fixed(1));
        next_btn.pane.set_at(DynVal::new_fixed(12), DynVal::new_fixed(1));

        // Add elements to pane (clones share Rc<RefCell<...>> state with originals)
        pane.add_element(Box::new(slider.clone()));
        pane.add_element(Box::new(prev_btn.clone()));
        pane.add_element(Box::new(play_pause_btn.clone()));
        pane.add_element(Box::new(stop_btn.clone()));
        pane.add_element(Box::new(next_btn.clone()));

        let mut player = Self {
            pane,
            sources: sources_rc,
            current_index,
            audio_ctx,
            stream,
            slider: Rc::new(RefCell::new(slider)),
            play_pause_btn: Rc::new(RefCell::new(play_pause_btn)),
            stop_btn: Rc::new(RefCell::new(stop_btn)),
            prev_btn: Rc::new(RefCell::new(prev_btn)),
            next_btn: Rc::new(RefCell::new(next_btn)),
        };

        // Load first track if sources provided
        {
            let sources = player.sources.borrow();
            if !sources.is_empty() {
                player.load_current();
            }
        }

        player
    }

    /// Refresh a button's visual content after text change.
    fn refresh_button(btn: &Rc<RefCell<Button>>, ctx: &Context) {
        let btn_ref = btn.borrow();
        let drawing = btn_ref.button_drawing(ctx);
        btn_ref.pane.set_content(drawing);
    }

    /// Set the audio sources and reset to the first track.
    pub fn set_sources(&self, sources: Vec<PathBuf>) {
        self.stop();
        *self.sources.borrow_mut() = sources;
        *self.current_index.borrow_mut() = 0;
        if !self.sources.borrow().is_empty() {
            self.load_current();
        }
    }

    /// Add a single audio file to the source list.
    pub fn add_source(&self, path: PathBuf) {
        self.sources.borrow_mut().push(path);
    }

    /// Load and prepare the currently selected track.
    fn load_current(&self) {
        let idx = *self.current_index.borrow();
        let sources = self.sources.borrow();
        if idx < sources.len() {
            AudioPlayer::load_track(
                &sources[idx],
                &self.audio_ctx,
                &self.stream,
                &self.play_pause_btn,
                &self.slider,
            );
        }
    }

    /// Decode a WAV file and set up the cpal stream for playback.
    fn load_track(
        path: &PathBuf,
        audio_ctx: &Rc<RefCell<Option<Arc<parking_lot::Mutex<AudioContext>>>>>,
        stream: &Rc<RefCell<Option<cpal::Stream>>>,
        play_pause_btn: &Rc<RefCell<Button>>,
        slider: &Rc<RefCell<Slider>>,
    ) {
        // Stop any existing playback
        if let Some(s) = stream.borrow().as_ref() {
            let _ = s.pause();
        }

        // Decode audio file (WAV, MP3, OGG, FLAC, AAC/M4A)
        let decoded = match Self::decode_audio(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to decode audio file {:?}: {}", path, e);
                return;
            }
        };

        let ac = Arc::new(parking_lot::Mutex::new(AudioContext {
            samples: decoded.samples,
            sample_rate: decoded.sample_rate,
            position: 0,
            state: AudioPlayerState::Stopped,
        }));

        // Create cpal stream
        let stream_result = Self::create_cpal_stream(&ac);
        match stream_result {
            Ok(s) => {
                *stream.borrow_mut() = Some(s);
                *audio_ctx.borrow_mut() = Some(ac);
                slider.borrow().set_position(0.0);
                *play_pause_btn.borrow_mut().text.borrow_mut() = "▶".to_string();
                // refresh_button skipped here — no Context available; UI updates next draw
            }
            Err(e) => {
                eprintln!("Failed to create audio stream: {}", e);
            }
        }
    }

    /// Decode an audio file using symphonia. Supports WAV, MP3, OGG, FLAC, AAC/M4A.
    /// Returns interleaved f32 samples held entirely in memory.
    fn decode_audio(path: &PathBuf) -> Result<DecodedAudio, String> {
        use symphonia::core::audio::{SampleBuffer, SignalSpec};
        use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        // Build a hint from the file extension
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav");
        let mut hint = Hint::new();
        hint.with_extension(ext);

        // Open the file
        let src = std::fs::File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        // Probe the media source
        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions::default();
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| format!("Failed to probe file (unsupported format?): {}", e))?;

        let mut format = probed.format;

        // Find the first audio track with a supported codec
        let track = format.tracks().iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| "No audio track found".to_string())?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate
            .ok_or_else(|| "Unknown sample rate".to_string())?;
        let _channels = track.codec_params.channels
            .map(|c| c.count())
            .unwrap_or(1);

        // Create a decoder for the track
        let dec_opts = DecoderOptions::default();
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .map_err(|e| format!("Failed to create decoder: {}", e))?;

        // Signal spec for the sample buffer (used for type conversion)
        let spec = SignalSpec::new(
            sample_rate,
            track.codec_params.channels.unwrap_or_default(),
        );

        let mut samples = Vec::new();

        // Decode all packets
        while let Ok(packet) = format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }
            let audio = match decoder.decode(&packet) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Decode error (skipping packet): {}", e);
                    continue;
                }
            };

            let n_frames = audio.frames();
            if n_frames == 0 {
                continue;
            }

            // Use SampleBuffer to convert any sample format to f32, interleaved
            let mut samp_buf = SampleBuffer::<f32>::new(n_frames as u64, spec);
            samp_buf.copy_interleaved_ref(audio);
            samples.extend_from_slice(samp_buf.samples());
        }

        Ok(DecodedAudio {
            samples,
            sample_rate,
        })
    }

    /// Create a cpal output stream for playback.
    fn create_cpal_stream(ctx: &Arc<parking_lot::Mutex<AudioContext>>) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| "No default output device found".to_string())?;

        let default_cfg = device.default_output_config()
            .map_err(|e| format!("No default output config: {}", e))?;

        // Build StreamConfig from default; cpal handles sample format conversion
        let config = cpal::StreamConfig {
            channels: default_cfg.channels(),
            sample_rate: default_cfg.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let ctx_clone = ctx.clone();

        let callback = move |data: &mut [f32], _: &_| {
            let mut audio = ctx_clone.lock();
            match audio.state {
                AudioPlayerState::Playing => {
                    let available = audio.samples.len().saturating_sub(audio.position);
                    let to_copy = available.min(data.len());
                    if to_copy > 0 {
                        let src = &audio.samples[audio.position..audio.position + to_copy];
                        data[..to_copy].copy_from_slice(src);
                    }
                    if to_copy < data.len() {
                        data[to_copy..].fill(0.0);
                        audio.state = AudioPlayerState::Stopped;
                        audio.position = 0;
                    } else {
                        audio.position += to_copy;
                    }
                }
                _ => {
                    data.fill(0.0);
                }
            }
        };

        let stream = device.build_output_stream(
            &config,
            callback,
            move |err: cpal::StreamError| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        ).map_err(|e| format!("Failed to build stream: {}", e))?;

        Ok(stream)
    }

    /// Start or resume playback.
    pub fn play(&self) {
        if let Some(ctx) = self.audio_ctx.borrow().as_ref() {
            let mut audio = ctx.lock();
            audio.state = AudioPlayerState::Playing;
        }
        if let Some(s) = self.stream.borrow().as_ref() {
            let _ = s.play();
        }
        *self.play_pause_btn.borrow_mut().text.borrow_mut() = "⏸".to_string();
        // refresh_button skipped — no Context available; UI updates next draw
    }

    /// Pause playback.
    pub fn pause(&self) {
        if let Some(ctx) = self.audio_ctx.borrow().as_ref() {
            let mut audio = ctx.lock();
            audio.state = AudioPlayerState::Paused;
        }
        if let Some(s) = self.stream.borrow().as_ref() {
            let _ = s.pause();
        }
        *self.play_pause_btn.borrow_mut().text.borrow_mut() = "▶".to_string();
        // refresh_button skipped — no Context available; UI updates next draw
    }

    /// Stop playback and reset position.
    pub fn stop(&self) {
        if let Some(ctx) = self.audio_ctx.borrow().as_ref() {
            let mut audio = ctx.lock();
            audio.state = AudioPlayerState::Stopped;
            audio.position = 0;
        }
        if let Some(s) = self.stream.borrow().as_ref() {
            let _ = s.pause();
        }
        *self.play_pause_btn.borrow_mut().text.borrow_mut() = "▶".to_string();
        self.slider.borrow().set_position(0.0);
        // refresh_button skipped — no Context available; UI updates next draw
    }

    /// Play previous track.
    pub fn prev(&self) {
        let mut idx = *self.current_index.borrow();
        let sources = self.sources.borrow();
        if sources.is_empty() {
            return;
        }
        if idx > 0 {
            idx -= 1;
        } else {
            idx = sources.len() - 1;
        }
        *self.current_index.borrow_mut() = idx;
        self.load_current();
        self.play();
    }

    /// Play next track.
    pub fn next(&self) {
        let mut idx = *self.current_index.borrow();
        let sources = self.sources.borrow();
        if sources.is_empty() {
            return;
        }
        idx = (idx + 1) % sources.len();
        *self.current_index.borrow_mut() = idx;
        self.load_current();
        self.play();
    }

    /// Get the current playback state.
    pub fn state(&self) -> AudioPlayerState {
        if let Some(ctx) = self.audio_ctx.borrow().as_ref() {
            ctx.lock().state
        } else {
            AudioPlayerState::Stopped
        }
    }

    /// Update slider position from audio context (called during drawing).
    fn update_slider_from_audio(&self) {
        if let Some(ctx) = self.audio_ctx.borrow().as_ref() {
            let audio = ctx.lock();
            let frac = audio.fraction();
            self.slider.borrow().set_position(frac);
        }
    }
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl AudioPlayer {
}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for AudioPlayer {
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        self.update_slider_from_audio();
        self.pane.drawing(ctx, dr, force_update)
    }
}

struct DecodedAudio {
    samples: Vec<f32>,
    sample_rate: u32,
}
