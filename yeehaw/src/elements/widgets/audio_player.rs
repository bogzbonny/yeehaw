use {
    crate::ch::{DrawCh, DrawChs2D},
    crate::color::Color,
    crate::context::Context,
    crate::draw_region::DrawRegion,
    crate::dyn_location::{DynLocation, DynLocationSet, Size, ZIndex},
    crate::dyn_value::DynVal,
    crate::element::{DrawUpdate, Element, Parent},
    crate::elements::panes::ParentPaneOfSelectable,
    crate::elements::widgets::button::{Button, ButtonMicroShadow},
    crate::elements::widgets::label::Label,
    crate::elements::widgets::slider::Slider,
    crate::event::{Event, EventResponses, ReceivableEvents},
    crate::sorting_hat::ElementID,
    crate::style::Style,
    crate::{Ref, RefMut},
    cpal::traits::{DeviceTrait, HostTrait, StreamTrait},
    std::cell::RefCell,
    std::path::PathBuf,
    std::rc::Rc,
    std::sync::Arc,
    std::sync::atomic::{AtomicBool, Ordering},
    std::sync::mpsc,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AudioPlayerState {
    Playing,
    Paused,
    Stopped,
}

/// Circular buffer for streaming decoded audio samples (interleaved f32).
/// Shared between the decoder thread (producer) and cpal callback (consumer).
struct RingBuffer {
    data: Vec<f32>,
    read_pos: usize,
    len: usize,
}

impl RingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            data: vec![0.0; capacity],
            read_pos: 0,
            len: 0,
        }
    }

    fn free_space(&self) -> usize {
        self.data.len() - self.len
    }

    /// Read up to `out.len()` samples. Returns number read.
    fn read(&mut self, out: &mut [f32]) -> usize {
        if self.len == 0 {
            return 0;
        }
        let to_read = out.len().min(self.len);
        let cap = self.data.len();

        let first_chunk = to_read.min(cap - self.read_pos);
        out[..first_chunk].copy_from_slice(&self.data[self.read_pos..self.read_pos + first_chunk]);

        if first_chunk < to_read {
            let second = to_read - first_chunk;
            out[first_chunk..].copy_from_slice(&self.data[..second]);
        }

        self.read_pos = (self.read_pos + to_read) % cap;
        self.len -= to_read;
        to_read
    }

    /// Write samples into the buffer. Returns number written.
    fn write(&mut self, samples: &[f32]) -> usize {
        if samples.is_empty() || self.len == self.data.len() {
            return 0;
        }
        let to_write = samples.len().min(self.free_space());
        let cap = self.data.len();
        let write_pos = (self.read_pos + self.len) % cap;

        let first_chunk = to_write.min(cap - write_pos);
        self.data[write_pos..write_pos + first_chunk].copy_from_slice(&samples[..first_chunk]);

        if first_chunk < to_write {
            let second = to_write - first_chunk;
            self.data[..second].copy_from_slice(&samples[first_chunk..to_write]);
        }

        self.len += to_write;
        to_write
    }

    fn clear(&mut self) {
        self.len = 0;
    }
}

/// Shared audio state accessible from the UI thread, cpal callback thread, and decoder thread.
struct AudioContext {
    ring_buf: Arc<parking_lot::Mutex<RingBuffer>>,
    sample_rate: u32,
    channels: u16,
    total_samples: usize,
    position: usize,
    state: AudioPlayerState,
    is_eof: bool,
}

impl AudioContext {
    fn fraction(&self) -> f64 {
        if self.total_samples == 0 {
            return 0.0;
        }
        self.position as f64 / self.total_samples as f64
    }
}

/// Command sent to the decoder thread.
enum DecoderCmd {
    /// Seek to the given time in seconds.
    Seek(f64),
}

/// Probed metadata from an audio file (no decoding).
struct ProbedAudio {
    duration_secs: f64,
    total_samples: usize,
    sample_rate: u32,
    channels: u16,
}

/// AudioPlayer element that plays audio files using cpal.
///
/// Uses streaming decode: a background thread decodes packets on-demand into a ring buffer.
/// The cpal callback reads from the ring buffer. Seeking is handled by seeking the format
/// reader in the decoder thread and flushing the ring buffer.
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

    // Decoder thread control
    decoder_handle: Rc<RefCell<Option<std::thread::JoinHandle<()>>>>,
    decoder_stop: Rc<RefCell<Option<Arc<AtomicBool>>>>,
    seek_tx: Rc<RefCell<Option<mpsc::Sender<DecoderCmd>>>>,

    // UI elements
    slider: Rc<RefCell<Slider>>,
    play_pause_btn: Rc<RefCell<Button>>,
    time_label: Rc<RefCell<Label>>,
    filename_label: Rc<RefCell<Label>>,
}

impl AudioPlayer {
    /// Create a new AudioPlayer with the given audio file paths.
    pub fn new(ctx: &Context, sources: Vec<PathBuf>) -> Self {
        let pane = ParentPaneOfSelectable::new(ctx).with_bg(Color::GREY10);
        pane.set_dyn_width(DynVal::FULL);
        pane.set_dyn_height(DynVal::new_fixed(2));

        // Create slider
        let slider = Slider::new_basic_line(ctx);
        slider.pane.set_dyn_width(DynVal::FULL);
        slider.pane.set_dyn_height(DynVal::new_fixed(1));
        slider
            .pane
            .set_at(DynVal::new_fixed(0), DynVal::new_fixed(0));
        slider.set_position(0.0);

        // Create buttons with micro shadow
        let shadow = ButtonMicroShadow::default();
        let prev_btn = Button::new(ctx, "⏮ ").with_micro_shadow(ctx, shadow.clone());
        let play_pause_btn = Button::new(ctx, "▶").with_micro_shadow(ctx, shadow.clone());
        let stop_btn = Button::new(ctx, "⏹").with_micro_shadow(ctx, shadow.clone());
        let next_btn = Button::new(ctx, "⏭ ").with_micro_shadow(ctx, shadow.clone());

        let sources_rc = Rc::new(RefCell::new(sources));
        let current_index = Rc::new(RefCell::new(0usize));
        let audio_ctx: Rc<RefCell<Option<Arc<parking_lot::Mutex<AudioContext>>>>> =
            Rc::new(RefCell::new(None));
        let stream: Rc<RefCell<Option<cpal::Stream>>> = Rc::new(RefCell::new(None));
        let decoder_handle: Rc<RefCell<Option<std::thread::JoinHandle<()>>>> =
            Rc::new(RefCell::new(None));
        let decoder_stop: Rc<RefCell<Option<Arc<AtomicBool>>>> = Rc::new(RefCell::new(None));
        let seek_tx: Rc<RefCell<Option<mpsc::Sender<DecoderCmd>>>> = Rc::new(RefCell::new(None));

        // Create labels for time and filename
        let time_label = Label::new(ctx, "0:00/0:00");
        let filename_label = Label::new(ctx, "");

        // Wire up slider callback for seeking (sends seek command to decoder thread)
        {
            let audio_ctx_clone = audio_ctx.clone();
            let seek_tx_clone = seek_tx.clone();
            slider.set_fn(Box::new(move |_: Context, sl: &Slider| {
                let frac = sl.get_position();
                let mut seek_secs = 0.0;
                if let Some(ac) = audio_ctx_clone.borrow().as_ref() {
                    let audio = ac.lock();
                    seek_secs = frac * audio.total_samples as f64
                        / (audio.sample_rate as f64 * audio.channels as f64);
                    drop(audio);
                    // Update position immediately for responsive UI
                    let mut audio = ac.lock();
                    audio.position = (frac * audio.total_samples as f64) as usize;
                    audio.is_eof = false;
                }
                // Tell decoder thread to seek
                if let Some(tx) = seek_tx_clone.borrow().as_ref() {
                    let _ = tx.send(DecoderCmd::Seek(seek_secs));
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
            let seek_tx_clone = seek_tx.clone();
            let play_pause_btn_clone = Rc::new(RefCell::new(play_pause_btn.clone()));

            stop_btn.set_fn(Box::new(move |_, ctx| {
                if let Some(ac) = audio_ctx_clone.borrow().as_ref() {
                    apply_stop(ac);
                }
                if let Some(s) = stream_clone.borrow().as_ref() {
                    let _ = s.pause();
                }
                // Seek decoder to beginning so next play starts from start of file
                if let Some(tx) = seek_tx_clone.borrow().as_ref() {
                    let _ = tx.send(DecoderCmd::Seek(0.0));
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
            let time_label_clone = Rc::new(RefCell::new(time_label.clone()));
            let filename_label_clone = Rc::new(RefCell::new(filename_label.clone()));
            let decoder_handle_clone = decoder_handle.clone();
            let decoder_stop_clone = decoder_stop.clone();
            let seek_tx_clone = seek_tx.clone();

            prev_btn.set_fn(Box::new(move |_, _| {
                // If playing and not at position 0, seek to beginning
                let should_seek = {
                    if let Some(ac) = audio_ctx_clone.borrow().as_ref() {
                        let audio = ac.lock();
                        audio.position > 0
                    } else {
                        false
                    }
                };
                if should_seek {
                    if let Some(ac) = audio_ctx_clone.borrow().as_ref() {
                        let mut audio = ac.lock();
                        audio.position = 0;
                        audio.is_eof = false;
                    }
                    slider_clone.borrow().set_position(0.0);
                    // Tell decoder to seek to beginning
                    if let Some(tx) = seek_tx_clone.borrow().as_ref() {
                        let _ = tx.send(DecoderCmd::Seek(0.0));
                    }
                    return EventResponses::default();
                }

                // Go to previous track
                let sources = sources_clone.borrow();
                if sources.len() <= 1 {
                    return EventResponses::default();
                }
                let mut idx = *current_index_clone.borrow();
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
                    &time_label_clone,
                    &filename_label_clone,
                    &decoder_handle_clone,
                    &decoder_stop_clone,
                    &seek_tx_clone,
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
            let time_label_clone = Rc::new(RefCell::new(time_label.clone()));
            let filename_label_clone = Rc::new(RefCell::new(filename_label.clone()));
            let decoder_handle_clone = decoder_handle.clone();
            let decoder_stop_clone = decoder_stop.clone();
            let seek_tx_clone = seek_tx.clone();

            next_btn.set_fn(Box::new(move |_, _| {
                let sources = sources_clone.borrow();
                if sources.len() <= 1 {
                    return EventResponses::default();
                }
                let mut idx = *current_index_clone.borrow();
                idx = (idx + 1) % sources.len();
                *current_index_clone.borrow_mut() = idx;
                AudioPlayer::load_track(
                    &sources[idx],
                    &audio_ctx_clone,
                    &stream_clone,
                    &play_pause_btn_clone,
                    &slider_clone,
                    &time_label_clone,
                    &filename_label_clone,
                    &decoder_handle_clone,
                    &decoder_stop_clone,
                    &seek_tx_clone,
                );
                EventResponses::default()
            }));
        }

        // Position buttons in row 1
        prev_btn
            .pane
            .set_at(DynVal::new_fixed(0), DynVal::new_fixed(1));
        play_pause_btn
            .pane
            .set_at(DynVal::new_fixed(4), DynVal::new_fixed(1));
        stop_btn
            .pane
            .set_at(DynVal::new_fixed(8), DynVal::new_fixed(1));
        next_btn
            .pane
            .set_at(DynVal::new_fixed(12), DynVal::new_fixed(1));

        // Position labels on row 1, right side
        time_label.set_at(DynVal::new_fixed(16), DynVal::new_fixed(1));
        filename_label.set_at(DynVal::new_fixed(26), DynVal::new_fixed(1));

        // Add elements to pane (clones share Rc<RefCell<...>> state with originals)
        pane.add_element(Box::new(slider.clone()));
        pane.add_element(Box::new(prev_btn.clone()));
        pane.add_element(Box::new(play_pause_btn.clone()));
        pane.add_element(Box::new(stop_btn.clone()));
        pane.add_element(Box::new(next_btn.clone()));
        pane.add_element(Box::new(time_label.clone()));
        pane.add_element(Box::new(filename_label.clone()));

        let player = Self {
            pane,
            sources: sources_rc,
            current_index,
            audio_ctx,
            stream,
            decoder_handle,
            decoder_stop,
            seek_tx,
            slider: Rc::new(RefCell::new(slider)),
            play_pause_btn: Rc::new(RefCell::new(play_pause_btn)),
            time_label: Rc::new(RefCell::new(time_label)),
            filename_label: Rc::new(RefCell::new(filename_label)),
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
    /// If this is the first source added, loads and prepares it for playback.
    pub fn add_source(&self, path: PathBuf) {
        let was_empty = self.sources.borrow().is_empty();
        self.sources.borrow_mut().push(path);
        if was_empty {
            *self.current_index.borrow_mut() = 0;
            self.load_current();
        }
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
                &self.time_label,
                &self.filename_label,
                &self.decoder_handle,
                &self.decoder_stop,
                &self.seek_tx,
            );
        }
    }

    /// Load a track using streaming decode. Probes metadata, spawns a decoder thread,
    /// and creates the cpal stream. Playback begins with zero pre-decoded samples.
    fn load_track(
        path: &PathBuf, audio_ctx: &Rc<RefCell<Option<Arc<parking_lot::Mutex<AudioContext>>>>>,
        stream: &Rc<RefCell<Option<cpal::Stream>>>, play_pause_btn: &Rc<RefCell<Button>>,
        slider: &Rc<RefCell<Slider>>, time_label: &Rc<RefCell<Label>>,
        filename_label: &Rc<RefCell<Label>>,
        decoder_handle: &Rc<RefCell<Option<std::thread::JoinHandle<()>>>>,
        decoder_stop: &Rc<RefCell<Option<Arc<AtomicBool>>>>,
        seek_tx: &Rc<RefCell<Option<mpsc::Sender<DecoderCmd>>>>,
    ) {
        // Stop previous decoder thread
        if let Some(stop_flag) = decoder_stop.borrow().as_ref() {
            stop_flag.store(true, Ordering::Relaxed);
        }
        if let Some(handle) = decoder_handle.borrow_mut().take() {
            let _ = handle.join();
        }
        // Pause previous stream
        if let Some(s) = stream.borrow().as_ref() {
            let _ = s.pause();
        }

        // Probe file metadata (fast — no decoding)
        let probe = match Self::probe_audio(path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to probe audio file {:?}: {}", path, e);
                return;
            }
        };

        let duration_secs = probe.duration_secs;
        let total_samples = probe.total_samples;

        // Ring buffer: ~5 seconds of audio capacity
        let rb_capacity = (probe.sample_rate as usize)
            .saturating_mul(probe.channels as usize)
            .saturating_mul(5);
        let ring_buf = Arc::new(parking_lot::Mutex::new(RingBuffer::new(
            rb_capacity.max(44100),
        )));

        let ac = Arc::new(parking_lot::Mutex::new(AudioContext {
            ring_buf: ring_buf.clone(),
            sample_rate: probe.sample_rate,
            channels: probe.channels,
            total_samples,
            position: 0,
            state: AudioPlayerState::Stopped,
            is_eof: false,
        }));

        // Create cpal stream (reads from ring buffer)
        let stream_result = Self::create_cpal_stream(&ac);
        match stream_result {
            Ok(s) => {
                *stream.borrow_mut() = Some(s);

                // Spawn decoder thread
                let (tx, rx) = mpsc::channel();
                let stop_flag = Arc::new(AtomicBool::new(false));
                let path_clone = path.clone();
                let rb = ring_buf.clone();
                let ac_clone = Arc::clone(&ac);

                let stop_flag_clone = Arc::clone(&stop_flag);
                let handle = std::thread::spawn(move || {
                    Self::decoder_thread(path_clone, rx, stop_flag_clone, rb, ac_clone);
                });

                *audio_ctx.borrow_mut() = Some(ac);
                *decoder_handle.borrow_mut() = Some(handle);
                *decoder_stop.borrow_mut() = Some(stop_flag);

                *seek_tx.borrow_mut() = Some(tx);

                slider.borrow().set_position(0.0);
                *play_pause_btn.borrow_mut().text.borrow_mut() = "▶".to_string();
                let duration_str = format_time(duration_secs);
                time_label
                    .borrow()
                    .set_text(format!("0:00/{}", duration_str));
                let name = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or("".into());
                filename_label.borrow().set_text(name.to_string());
            }
            Err(e) => {
                eprintln!("Failed to create audio stream: {}", e);
            }
        }
    }

    /// Probe an audio file for metadata without decoding. Uses n_frames from track info
    /// if available; otherwise does a fast packet scan to count frames.
    fn probe_audio(path: &PathBuf) -> Result<ProbedAudio, String> {
        use symphonia::core::codecs::CODEC_TYPE_NULL;
        use symphonia::core::formats::FormatOptions;
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("wav");
        let mut hint = Hint::new();
        hint.with_extension(ext);

        let src = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions::default();
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| format!("Failed to probe file: {}", e))?;

        let format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| "No audio track found".to_string())?;

        let sample_rate = track
            .codec_params
            .sample_rate
            .ok_or_else(|| "Unknown sample rate".to_string())?;
        let channels: u16 = track
            .codec_params
            .channels
            .map(|c| c.count() as u16)
            .unwrap_or(1);

        // Try to get total frames from metadata
        let total_frames = track.codec_params.n_frames;

        let (duration_secs, total_samples) = if let Some(n_frames) = total_frames {
            let dur = n_frames as f64 / sample_rate as f64;
            let total = (n_frames as usize).saturating_mul(channels as usize);
            (dur, total)
        } else {
            // Fast scan: iterate packets and sum frame counts without full decode
            let dec_opts = symphonia::core::codecs::DecoderOptions::default();
            let mut decoder = symphonia::default::get_codecs()
                .make(&track.codec_params, &dec_opts)
                .map_err(|e| format!("Failed to create decoder: {}", e))?;
            let track_id = track.id;
            let _spec = symphonia::core::audio::SignalSpec::new(
                sample_rate,
                track.codec_params.channels.unwrap_or_default(),
            );
            let mut total_frames: usize = 0;
            let mut fmt = format;
            while let Ok(packet) = fmt.next_packet() {
                if packet.track_id() != track_id {
                    continue;
                }
                match decoder.decode(&packet) {
                    Ok(audio) => total_frames += audio.frames(),
                    Err(_) => continue,
                }
            }
            let dur = total_frames as f64 / sample_rate as f64;
            let total = total_frames.saturating_mul(channels as usize);
            (dur, total)
        };

        Ok(ProbedAudio {
            duration_secs,
            total_samples,
            sample_rate,
            channels,
        })
    }

    /// Background decoder thread. Reads packets from the file, decodes them, and writes
    /// interleaved f32 samples into the ring buffer. Listens for seek/stop commands.
    fn decoder_thread(
        path: PathBuf, cmd_rx: mpsc::Receiver<DecoderCmd>, stop_flag: Arc<AtomicBool>,
        ring_buf: Arc<parking_lot::Mutex<RingBuffer>>,
        audio_ctx: Arc<parking_lot::Mutex<AudioContext>>,
    ) {
        use symphonia::core::audio::{SampleBuffer, SignalSpec};
        use symphonia::core::codecs::DecoderOptions;
        use symphonia::core::formats::{FormatOptions, SeekMode};
        use symphonia::core::io::MediaSourceStream;
        use symphonia::core::meta::MetadataOptions;
        use symphonia::core::probe::Hint;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("wav");
        let mut hint = Hint::new();
        hint.with_extension(ext);

        let src = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Decoder thread: failed to open {:?}: {}", path, e);
                return;
            }
        };
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions::default();
        let probed = match symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)
        {
            Ok(p) => p.format,
            Err(e) => {
                eprintln!("Decoder thread: failed to probe: {}", e);
                return;
            }
        };

        use symphonia::core::codecs::CODEC_TYPE_NULL;
        let track = probed
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no audio track");
        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.expect("unknown sample rate");
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(1);
        let spec = SignalSpec::new(sample_rate, track.codec_params.channels.unwrap_or_default());
        // Clone codec_params before moving probed (needed for decoder recreation after seek)
        let codec_params_clone = track.codec_params.clone();

        let dec_opts = DecoderOptions::default();
        let mut format = probed;
        let mut decoder =
            match symphonia::default::get_codecs().make(&codec_params_clone, &dec_opts) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Decoder thread: failed to create decoder: {}", e);
                    return;
                }
            };

        loop {
            // Check for commands (non-blocking)
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    DecoderCmd::Seek(time_secs) => {
                        // Seek to timestamp
                        let ts = (time_secs * sample_rate as f64) as u64;
                        if let Ok(_seeked) = format.seek(
                            SeekMode::Accurate,
                            symphonia::core::formats::SeekTo::TimeStamp { ts, track_id },
                        ) {
                            // Recreate decoder after seek (required by symphonia)
                            if let Ok(new_dec) = symphonia::default::get_codecs()
                                .make(&codec_params_clone, &dec_opts)
                            {
                                decoder = new_dec;
                            }
                            // Clear ring buffer
                            ring_buf.lock().clear();
                            // Update position — use requested time (keyframe alignment may cause small offset)
                            let pos_samples =
                                (time_secs * sample_rate as f64 * channels as f64) as usize;
                            let mut audio = audio_ctx.lock();
                            audio.position = pos_samples;
                            audio.is_eof = false;
                        }
                    }
                }
            }

            if stop_flag.load(Ordering::Relaxed) {
                return;
            }

            // Back-pressure: don't decode unless the buffer has meaningful room.
            // Keeps 20% of the buffer free so decoded packets aren't partially dropped.
            {
                let rb = ring_buf.lock();
                let threshold = rb.data.len() / 5;
                if rb.free_space() < threshold {
                    drop(rb);
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
            }

            match format.next_packet() {
                Ok(packet) => {
                    if packet.track_id() != track_id {
                        continue;
                    }
                    match decoder.decode(&packet) {
                        Ok(audio) => {
                            let n_frames = audio.frames();
                            if n_frames == 0 {
                                continue;
                            }
                            let mut samp_buf = SampleBuffer::<f32>::new(n_frames as u64, spec);
                            samp_buf.copy_interleaved_ref(audio);
                            ring_buf.lock().write(samp_buf.samples());
                        }
                        Err(e) => {
                            eprintln!("Decoder thread: decode error: {}", e);
                        }
                    }
                }
                Err(_) => {
                    // End of stream
                    audio_ctx.lock().is_eof = true;
                    return;
                }
            }
        }
    }

    /// Create a cpal output stream that reads from the ring buffer.
    fn create_cpal_stream(
        ctx: &Arc<parking_lot::Mutex<AudioContext>>,
    ) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default output device found".to_string())?;

        let (sample_rate, channels) = {
            let audio = ctx.lock();
            (audio.sample_rate, audio.channels)
        };

        let config = cpal::StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let ctx_clone = ctx.clone();

        let callback = move |data: &mut [f32], _: &_| {
            let mut audio = ctx_clone.lock();
            match audio.state {
                AudioPlayerState::Playing => {
                    let copied = audio.ring_buf.lock().read(data);
                    audio.position += copied;
                    if copied < data.len() {
                        data[copied..].fill(0.0);
                        // End of playback: ring buffer empty and decoder finished
                        if audio.ring_buf.lock().len == 0 && audio.is_eof {
                            audio.state = AudioPlayerState::Stopped;
                            audio.position = 0;
                        }
                    }
                }
                _ => {
                    data.fill(0.0);
                }
            }
        };

        let stream = device
            .build_output_stream(
                &config,
                callback,
                move |err: cpal::StreamError| {
                    eprintln!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| format!("Failed to build stream: {}", e))?;

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
            apply_stop(ctx);
        }
        if let Some(s) = self.stream.borrow().as_ref() {
            let _ = s.pause();
        }
        // Seek decoder to beginning so next play starts from start of file
        if let Some(tx) = self.seek_tx.borrow().as_ref() {
            let _ = tx.send(DecoderCmd::Seek(0.0));
        }
        *self.play_pause_btn.borrow_mut().text.borrow_mut() = "▶".to_string();
        self.slider.borrow().set_position(0.0);
        // refresh_button skipped — no Context available; UI updates next draw
    }

    /// Play previous track.
    pub fn prev(&self) {
        let sources = self.sources.borrow();
        if sources.len() <= 1 {
            return;
        }
        let mut idx = *self.current_index.borrow();
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
        let sources = self.sources.borrow();
        if sources.len() <= 1 {
            return;
        }
        let mut idx = *self.current_index.borrow();
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

    /// Set the position of this AudioPlayer (builder pattern).
    pub fn at<D: Into<DynVal>, D2: Into<DynVal>>(self, x: D, y: D2) -> Self {
        self.pane.set_at(x.into(), y.into());
        self
    }

    /// Set the position of this AudioPlayer.
    pub fn set_at(&self, x: DynVal, y: DynVal) {
        self.pane.set_at(x, y);
    }

    /// Update slider position and play/pause button text from audio context (called during drawing).
    fn update_slider_from_audio(&self) {
        if let Some(ctx) = self.audio_ctx.borrow().as_ref() {
            let audio = ctx.lock();
            self.slider.borrow().set_position(audio.fraction());
            // Sync button text with actual state (state can change in cpal callback)
            let icon = match audio.state {
                AudioPlayerState::Playing => "⏸",
                _ => "▶",
            };
            *self.play_pause_btn.borrow_mut().text.borrow_mut() = icon.to_string();
            // Update time label
            let samples_per_sec = audio.sample_rate as f64 * audio.channels as f64;
            let current_secs = audio.position as f64 / samples_per_sec;
            let duration_secs = audio.total_samples as f64 / samples_per_sec;
            let time_str = format!(
                "{}/{}",
                format_time(current_secs),
                format_time(duration_secs)
            );
            self.time_label.borrow().set_text(time_str);
        }
    }
}

#[yeehaw_derive::impl_pane_basics_from(pane)]
impl AudioPlayer {}

#[yeehaw_derive::impl_element_from(pane)]
impl Element for AudioPlayer {
    fn drawing(&self, ctx: &Context, dr: &DrawRegion, force_update: bool) -> Vec<DrawUpdate> {
        self.update_slider_from_audio();
        self.pane.drawing(ctx, dr, force_update)
    }
}

/// Format seconds as "m:ss" (e.g. "3:14").
fn format_time(secs: f64) -> String {
    let total_secs = secs as u32;
    let mins = total_secs / 60;
    let s = total_secs % 60;
    format!("{}:{:02}", mins, s)
}

/// Apply the stop operation to shared audio state.
/// Called by both the public `stop()` method and the stop button handler.
fn apply_stop(audio_ctx: &Arc<parking_lot::Mutex<AudioContext>>) {
    let mut audio = audio_ctx.lock();
    audio.state = AudioPlayerState::Stopped;
    audio.position = 0;
    audio.ring_buf.lock().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_clears_ring_buffer_and_resets_position() {
        let ring_buf = Arc::new(parking_lot::Mutex::new(RingBuffer::new(1024)));
        let audio_ctx = Arc::new(parking_lot::Mutex::new(AudioContext {
            ring_buf: ring_buf.clone(),
            sample_rate: 44100,
            channels: 2,
            total_samples: 10000,
            position: 5000,
            state: AudioPlayerState::Playing,
            is_eof: false,
        }));

        // Write data to the ring buffer (simulating decoded audio mid-playback)
        let samples = vec![1.0f32; 100];
        ring_buf.lock().write(&samples);
        assert_eq!(ring_buf.lock().len, 100);
        assert_eq!(audio_ctx.lock().position, 5000);

        // Perform stop via the shared stop logic
        apply_stop(&audio_ctx);

        // After stop, the ring buffer must be empty and position reset to 0
        assert_eq!(
            ring_buf.lock().len,
            0,
            "ring buffer should be cleared after stop"
        );
        assert_eq!(
            audio_ctx.lock().position,
            0,
            "position should be reset to 0 after stop"
        );
        assert_eq!(
            audio_ctx.lock().state,
            AudioPlayerState::Stopped,
            "state should be Stopped"
        );
    }

    #[test]
    fn test_stop_then_play_starts_from_beginning() {
        let ring_buf = Arc::new(parking_lot::Mutex::new(RingBuffer::new(1024)));
        let audio_ctx = Arc::new(parking_lot::Mutex::new(AudioContext {
            ring_buf: ring_buf.clone(),
            sample_rate: 44100,
            channels: 2,
            total_samples: 10000,
            position: 0,
            state: AudioPlayerState::Stopped,
            is_eof: false,
        }));

        // Simulate playback: decoder writes data, cpal callback reads and advances position
        let mid_data = vec![9.0f32; 50];
        ring_buf.lock().write(&mid_data);
        {
            let mut audio = audio_ctx.lock();
            audio.state = AudioPlayerState::Playing;
            audio.position = 3000;
        }
        // Cpal callback drains some data
        let mut out = vec![0.0f32; 30];
        ring_buf.lock().read(&mut out);
        {
            let mut audio = audio_ctx.lock();
            audio.position += 30;
        }

        // Now stop
        apply_stop(&audio_ctx);

        // After stop, buffer must be empty so play starts fresh
        assert_eq!(ring_buf.lock().len, 0, "buffer must be empty after stop");
        assert_eq!(
            audio_ctx.lock().position,
            0,
            "position must be 0 after stop"
        );

        // Simulate play: decoder writes data from the beginning
        let start_data = vec![1.0f32; 50];
        ring_buf.lock().write(&start_data);
        {
            let mut audio = audio_ctx.lock();
            audio.state = AudioPlayerState::Playing;
        }

        // Cpal callback reads — should get data from the beginning, not stale mid-data
        let mut out = vec![0.0f32; 50];
        ring_buf.lock().read(&mut out);
        assert!(
            out.iter().all(|&x| (x - 1.0).abs() < f32::EPSILON),
            "playback after stop should start from beginning data, not stale mid-playback data"
        );
    }
}
