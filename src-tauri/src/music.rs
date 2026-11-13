//! Music library sync: scan a folder on the PC (tags, covers, duration),
//! transcode to MP3 64 kbps CBR 44.1 kHz with LAME, and sync the result to the
//! watch's hidden partition under `data/mass/music/` (read by the watch music
//! app as `internal://mass/music/`).
//!
//! Layout on the watch (a contract with the watch app):
//! - `index.json`: version, generated time, tracks and playlists.
//! - `tracks/<id>.mp3`: one file per track, `id` = 8 hex chars.
//! - `covers/<id>.jpg`: about 160x160 baseline JPEG, `id` = hash of the image
//!   bytes so tracks of one album share one file.
//!
//! This module has no Tauri dependency; progress goes through a callback.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

/// Output bitrate in bytes per second (64 kbps).
pub const BYTES_PER_SEC: f64 = 8000.0;
/// Rough size of one encoded cover (160x160 JPEG, quality 80).
pub const COVER_EST: u64 = 8 * 1024;
/// Space kept free on the watch drive after a sync.
pub const MARGIN: u64 = 10 * 1024 * 1024;
/// Side of the cover thumbnails.
const COVER_SIDE: u32 = 160;
/// Sources that are already MP3 at 44.1 kHz and at most this bitrate are
/// copied instead of re-encoded.
const COPY_MAX_KBPS: u32 = 80;

const AUDIO_EXTS: &[&str] = &["mp3", "flac", "wav", "ogg", "oga", "m4a", "aac", "aif", "aiff", "mp4"];
const COVER_NAMES: &[&str] = &["cover.jpg", "folder.jpg", "front.jpg", "cover.jpeg", "folder.jpeg", "cover.png", "folder.png", "front.png"];

/// The music folder on the hidden partition.
pub fn music_dir(root: &Path) -> PathBuf {
    root.join("data").join("mass").join("music")
}

// ---------------------------------------------------------------------------
// Pure helpers (unit tested)
// ---------------------------------------------------------------------------

/// The first 4 bytes of the MD5 of `data`, as 8 lowercase hex chars.
fn short_hash(data: &[u8]) -> String {
    let d = Md5::digest(data);
    d[..4].iter().map(|b| format!("{b:02x}")).collect()
}

/// Stable id of a source file: hash of its normalized relative path (lowercase,
/// "/"-separated), its size and its modification time (seconds).
pub fn track_id(rel: &str, size: u64, mtime: u64) -> String {
    let norm = rel.replace('\\', "/").to_lowercase();
    short_hash(format!("{norm}|{size}|{mtime}").as_bytes())
}

/// Id of a cover image: hash of its bytes, so identical art is shared.
pub fn cover_id(bytes: &[u8]) -> String {
    short_hash(bytes)
}

/// For a file under `root`, its "/"-separated relative path and the relative
/// path of its folder ("" for files directly in the root).
pub fn rel_and_folder(root: &Path, file: &Path) -> Option<(String, String)> {
    let rel = file.strip_prefix(root).ok()?;
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.is_empty() {
        return None;
    }
    let folder = parts[..parts.len() - 1].join("/");
    Some((parts.join("/"), folder))
}

/// Estimated encoded size of `duration` seconds at 64 kbps.
pub fn estimate_size(duration: f64) -> u64 {
    (duration.max(0.0) * BYTES_PER_SEC).ceil() as u64
}

/// Lowercase "/"-separated path with "." and ".." resolved lexically.
fn normalize_rel(p: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for part in p.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            s => out.push(s),
        }
    }
    out.join("/").to_lowercase()
}

/// Map the entries of an M3U/M3U8 playlist to track ids. `dir_rel` is the
/// playlist's folder relative to the library root, `root` is the library root
/// (for absolute entries), and `lookup` maps normalized relative paths to ids.
pub fn parse_m3u(text: &str, dir_rel: &str, root: &Path, lookup: &HashMap<String, String>) -> Vec<String> {
    let root_norm = normalize_rel(&root.to_string_lossy());
    let mut ids = Vec::new();
    for line in text.trim_start_matches('\u{feff}').lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let entry = line.strip_prefix("file:///").unwrap_or(line);
        let entry = entry.replace("%20", " ");
        let bytes = entry.as_bytes();
        let absolute = (bytes.len() > 2 && bytes[1] == b':')
            || entry.starts_with('/')
            || entry.starts_with('\\');
        let rel = if absolute {
            let n = normalize_rel(&entry);
            match n.strip_prefix(&root_norm) {
                Some(r) => r.trim_start_matches('/').to_string(),
                None => continue,
            }
        } else {
            normalize_rel(&format!("{dir_rel}/{entry}"))
        };
        if let Some(id) = lookup.get(&rel) {
            ids.push(id.clone());
        }
    }
    ids
}

// ---------------------------------------------------------------------------
// index.json
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IndexTrack {
    pub id: String,
    pub file: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub folder: String,
    pub track: u32,
    pub duration: f64,
    pub cover: String,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Index {
    pub version: u32,
    pub generated: String,
    pub tracks: Vec<IndexTrack>,
    pub playlists: Vec<Playlist>,
}

/// Build the index: tracks sorted by folder, track number, title; playlists
/// keep only tracks present in the index and are dropped when empty.
pub fn build_index(mut tracks: Vec<IndexTrack>, playlists: &[Playlist], generated: &str) -> Index {
    tracks.sort_by(|a, b| {
        a.folder
            .to_lowercase()
            .cmp(&b.folder.to_lowercase())
            .then(a.track.cmp(&b.track))
            .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });
    let present: HashSet<&str> = tracks.iter().map(|t| t.id.as_str()).collect();
    let playlists = playlists
        .iter()
        .map(|p| Playlist {
            name: p.name.clone(),
            tracks: p.tracks.iter().filter(|id| present.contains(id.as_str())).cloned().collect(),
        })
        .filter(|p| !p.tracks.is_empty())
        .collect();
    Index { version: 1, generated: generated.to_string(), tracks, playlists }
}

/// Repair Hebrew tags stored as Windows-1255 bytes but read as Latin-1 (common
/// in old ID3 tags): "àäáä" becomes "אהבה". Only words made entirely of
/// characters in the Hebrew letter range (0xE0..0xFA) are converted, so Latin
/// words with accents ("café") stay as they are.
pub fn fix_hebrew(s: &str) -> String {
    let heb = |c: char| ('\u{e0}'..='\u{fa}').contains(&c);
    let word_ok = |w: &str| {
        let letters: Vec<char> = w.chars().filter(|c| c.is_alphabetic()).collect();
        !letters.is_empty() && letters.iter().all(|c| heb(*c))
    };
    let has_high = s.chars().any(|c| (c as u32) >= 0x80);
    let all_latin1 = s.chars().all(|c| (c as u32) < 0x100);
    if !has_high || !all_latin1 {
        return s.to_string();
    }
    let words: Vec<&str> = s.split_whitespace().filter(|w| w.chars().any(|c| (c as u32) >= 0x80)).collect();
    if !words.iter().all(|w| word_ok(w)) {
        return s.to_string();
    }
    s.chars()
        .map(|c| if heb(c) { char::from_u32(0x05d0 + (c as u32 - 0xe0)).unwrap_or(c) } else { c })
        .collect()
}

fn now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Write `index.json` atomically (temp file, then rename over the old one).
fn write_index(dir: &Path, index: &Index) -> Result<(), String> {
    let text = serde_json::to_vec_pretty(index).map_err(|e| e.to_string())?;
    let tmp = dir.join("index.json.tmp");
    {
        let mut f = File::create(&tmp).map_err(|e| format!("index: {e}"))?;
        f.write_all(&text).map_err(|e| format!("index: {e}"))?;
        f.sync_all().ok();
    }
    std::fs::rename(&tmp, dir.join("index.json")).map_err(|e| format!("index: {e}"))
}

/// Read the index currently on the watch (empty if none or unreadable).
pub fn read_index(root: &Path) -> Index {
    std::fs::read(music_dir(root).join("index.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Scan
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum CoverSrc {
    None,
    Embedded,
    File(PathBuf),
}

#[derive(Clone, Debug)]
pub struct ScanTrack {
    pub id: String,
    pub rel: String,
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub folder: String,
    pub track: u32,
    pub duration: f64,
    pub est_size: u64,
    pub cover: String,
    cover_src: CoverSrc,
    copy_as_is: bool,
}

/// What the UI sees of a scanned track.
#[derive(Serialize, Clone)]
pub struct TrackOut {
    pub id: String,
    pub rel: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub folder: String,
    pub track: u32,
    pub duration: f64,
    pub est_size: u64,
    pub cover: String,
}

#[derive(Serialize, Clone)]
pub struct ScanOut {
    pub root: String,
    pub tracks: Vec<TrackOut>,
    pub playlists: Vec<Playlist>,
    pub cover_est: u64,
    /// Audio files that could not be read.
    pub skipped: usize,
}

struct Scan {
    tracks: Vec<ScanTrack>,
    playlists: Vec<Playlist>,
}

fn last_scan() -> &'static Mutex<Option<Scan>> {
    static S: OnceLock<Mutex<Option<Scan>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

fn cancel_flag() -> &'static AtomicBool {
    static C: AtomicBool = AtomicBool::new(false);
    &C
}

/// Ask a running scan or sync to stop at the next file.
pub fn cancel() {
    cancel_flag().store(true, Ordering::SeqCst);
}

fn ext_of(p: &Path) -> String {
    p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
}

/// All files under `dir`, recursively, sorted, skipping hidden/system folders.
fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if name.starts_with('.') || name.starts_with('$') || name == "System Volume Information" {
            continue;
        }
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

/// Folder cover file next to a track, if any.
fn folder_cover(dir: &Path) -> Option<PathBuf> {
    let rd = std::fs::read_dir(dir).ok()?;
    let files: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
    for want in COVER_NAMES {
        if let Some(p) = files.iter().find(|p| {
            p.file_name().map(|n| n.to_string_lossy().to_lowercase()).as_deref() == Some(*want)
        }) {
            return Some(p.clone());
        }
    }
    None
}

/// Duration via symphonia when lofty cannot read the file.
fn probe_duration(path: &Path) -> Option<f64> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    let mss = MediaSourceStream::new(Box::new(File::open(path).ok()?), Default::default());
    let mut hint = Hint::new();
    hint.with_extension(&ext_of(path));
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .ok()?;
    let t = probed.format.default_track()?;
    let frames = t.codec_params.n_frames? as f64;
    let rate = t.codec_params.sample_rate? as f64;
    Some(frames / rate)
}

/// Read one audio file. Returns None when it cannot be decoded at all.
fn scan_file(root: &Path, path: &Path, folder_covers: &mut HashMap<PathBuf, Option<(PathBuf, String)>>) -> Option<ScanTrack> {
    use lofty::prelude::*;
    use lofty::picture::PictureType;

    let (rel, folder) = rel_and_folder(root, path)?;
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let id = track_id(&rel, meta.len(), mtime);
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();

    let (mut title, mut artist, mut album, mut track) = (String::new(), String::new(), String::new(), 0u32);
    let duration: f64;
    let mut cover = String::new();
    let mut cover_src = CoverSrc::None;
    let mut copy_as_is = false;

    match lofty::read_from_path(path) {
        Ok(tf) => {
            let props = tf.properties();
            duration = props.duration().as_secs_f64();
            if ext_of(path) == "mp3"
                && props.sample_rate() == Some(44100)
                && props.audio_bitrate().is_some_and(|b| b > 0 && b <= COPY_MAX_KBPS)
            {
                copy_as_is = true;
            }
            if let Some(tag) = tf.primary_tag().or_else(|| tf.first_tag()) {
                let clean = |s: Option<std::borrow::Cow<'_, str>>| s.map(|s| fix_hebrew(s.trim())).unwrap_or_default();
                title = clean(tag.title());
                artist = clean(tag.artist());
                album = clean(tag.album());
                track = tag.track().unwrap_or(0);
            }
            for tag in tf.tags() {
                let pic = tag
                    .get_picture_type(PictureType::CoverFront)
                    .or_else(|| tag.pictures().first());
                if let Some(p) = pic {
                    if !p.data().is_empty() {
                        cover = cover_id(p.data());
                        cover_src = CoverSrc::Embedded;
                        break;
                    }
                }
            }
        }
        Err(_) => {
            duration = probe_duration(path)?;
        }
    }
    if title.is_empty() {
        title = stem;
    }
    if cover.is_empty() {
        let dir = path.parent()?.to_path_buf();
        let entry = folder_covers.entry(dir.clone()).or_insert_with(|| {
            let f = folder_cover(&dir)?;
            let bytes = std::fs::read(&f).ok()?;
            Some((f, cover_id(&bytes)))
        });
        if let Some((f, cid)) = entry {
            cover = cid.clone();
            cover_src = CoverSrc::File(f.clone());
        }
    }
    let est_size = if copy_as_is { meta.len() } else { estimate_size(duration) };
    Some(ScanTrack {
        id,
        rel,
        path: path.to_path_buf(),
        title,
        artist,
        album,
        folder,
        track,
        duration: (duration * 10.0).round() / 10.0,
        est_size,
        cover,
        cover_src,
        copy_as_is,
    })
}

/// Scan a library folder. `progress(done, total)` is called as files are read.
/// The result is kept for a later `sync`.
pub fn scan(root: &Path, progress: &dyn Fn(usize, usize)) -> Result<ScanOut, String> {
    if !root.is_dir() {
        return Err("folder_not_found".into());
    }
    cancel_flag().store(false, Ordering::SeqCst);
    let mut files = Vec::new();
    walk(root, &mut files);
    let audio: Vec<&PathBuf> = files.iter().filter(|p| AUDIO_EXTS.contains(&ext_of(p).as_str())).collect();
    let total = audio.len();
    let mut tracks = Vec::new();
    let mut skipped = 0;
    let mut folder_covers = HashMap::new();
    for (i, p) in audio.iter().enumerate() {
        if cancel_flag().load(Ordering::SeqCst) {
            return Err("cancelled".into());
        }
        match scan_file(root, p, &mut folder_covers) {
            Some(t) => tracks.push(t),
            None => skipped += 1,
        }
        if i % 10 == 0 || i + 1 == total {
            progress(i + 1, total);
        }
    }

    // Playlists from .m3u / .m3u8 files.
    let lookup: HashMap<String, String> =
        tracks.iter().map(|t| (normalize_rel(&t.rel), t.id.clone())).collect();
    let mut playlists = Vec::new();
    for p in files.iter().filter(|p| matches!(ext_of(p).as_str(), "m3u" | "m3u8")) {
        let Some((_, dir_rel)) = rel_and_folder(root, p) else { continue };
        let Ok(bytes) = std::fs::read(p) else { continue };
        let text = String::from_utf8_lossy(&bytes);
        let ids = parse_m3u(&text, &dir_rel, root, &lookup);
        if !ids.is_empty() {
            let name = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            playlists.push(Playlist { name, tracks: ids });
        }
    }

    let out = ScanOut {
        root: root.to_string_lossy().into_owned(),
        tracks: tracks
            .iter()
            .map(|t| TrackOut {
                id: t.id.clone(),
                rel: t.rel.clone(),
                title: t.title.clone(),
                artist: t.artist.clone(),
                album: t.album.clone(),
                folder: t.folder.clone(),
                track: t.track,
                duration: t.duration,
                est_size: t.est_size,
                cover: t.cover.clone(),
            })
            .collect(),
        playlists: playlists.clone(),
        cover_est: COVER_EST,
        skipped,
    };
    *last_scan().lock().map_err(|e| e.to_string())? = Some(Scan { tracks, playlists });
    Ok(out)
}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

fn new_encoder(in_rate: u32) -> Result<mp3lame_encoder::Encoder, String> {
    use mp3lame_encoder::{Bitrate, Builder, Mode, Quality};
    let mut b = Builder::new().ok_or("lame_init")?;
    b.set_num_channels(2).map_err(|e| format!("lame: {e:?}"))?;
    b.set_sample_rate(in_rate).map_err(|e| format!("lame: {e:?}"))?;
    b.set_output_sample_rate(std::num::NonZeroU32::new(44100)).map_err(|e| format!("lame: {e:?}"))?;
    b.set_brate(Bitrate::Kbps64).map_err(|e| format!("lame: {e:?}"))?;
    b.set_mode(Mode::JointStereo).map_err(|e| format!("lame: {e:?}"))?;
    b.set_quality(Quality::Good).map_err(|e| format!("lame: {e:?}"))?;
    b.set_to_write_vbr_tag(false).map_err(|e| format!("lame: {e:?}"))?;
    b.build().map_err(|e| format!("lame: {e:?}"))
}

/// Feed one block of planar stereo samples (-1.0..1.0) to the encoder.
fn encode_block(enc: &mut mp3lame_encoder::Encoder, l: &[f32], r: &[f32], out: &mut Vec<u8>) -> Result<(), String> {
    out.reserve(mp3lame_encoder::max_required_buffer_size(l.len()));
    enc.encode_to_vec(mp3lame_encoder::DualPcm { left: l, right: r }, out)
        .map_err(|e| format!("encode: {e:?}"))?;
    Ok(())
}

fn finish(enc: &mut mp3lame_encoder::Encoder, out: &mut Vec<u8>) -> Result<(), String> {
    out.reserve(7200);
    enc.flush_to_vec::<mp3lame_encoder::FlushNoGap>(out)
        .map_err(|e| format!("flush: {e:?}"))?;
    Ok(())
}

/// Encode planar stereo PCM at `rate` to MP3 64 kbps 44.1 kHz (used by tests
/// and small inputs).
#[cfg(test)]
pub fn encode_pcm(left: &[f32], right: &[f32], rate: u32) -> Result<Vec<u8>, String> {
    let mut enc = new_encoder(rate)?;
    let mut out = Vec::new();
    for (l, r) in left.chunks(8192).zip(right.chunks(8192)) {
        encode_block(&mut enc, l, r, &mut out)?;
    }
    finish(&mut enc, &mut out)?;
    Ok(out)
}

/// Decode any supported source and encode it to MP3 bytes.
pub fn transcode(src: &Path, cancel: &AtomicBool) -> Result<Vec<u8>, String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
    use symphonia::core::errors::Error as SErr;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = File::open(src).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    hint.with_extension(&ext_of(src));
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("unsupported: {e}"))?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("no_audio_track")?;
    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("unsupported: {e}"))?;

    let mut enc: Option<mp3lame_encoder::Encoder> = None;
    let mut out = Vec::new();
    let (mut l, mut r) = (Vec::new(), Vec::new());
    let mut buf: Option<SampleBuffer<f32>> = None;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SErr::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SErr::ResetRequired) => break,
            Err(e) => return Err(format!("read: {e}")),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(SErr::DecodeError(_)) | Err(SErr::IoError(_)) => continue,
            Err(e) => return Err(format!("decode: {e}")),
        };
        let spec = *decoded.spec();
        let ch = spec.channels.count().max(1);
        if enc.is_none() {
            enc = Some(new_encoder(spec.rate)?);
        }
        let need = decoded.capacity() as u64;
        if buf.as_ref().is_none_or(|b| b.capacity() < need as usize * ch) {
            buf = Some(SampleBuffer::<f32>::new(need, spec));
        }
        let b = buf.as_mut().expect("buffer");
        b.copy_interleaved_ref(decoded);
        let s = b.samples();
        l.clear();
        r.clear();
        for frame in s.chunks_exact(ch) {
            l.push(frame[0]);
            r.push(if ch > 1 { frame[1] } else { frame[0] });
        }
        encode_block(enc.as_mut().expect("encoder"), &l, &r, &mut out)?;
    }
    let mut enc = enc.ok_or("no_audio")?;
    finish(&mut enc, &mut out)?;
    Ok(out)
}

/// An MP3 copied as is, without its ID3v2 / ID3v1 tags (they may carry large
/// pictures the watch does not need).
fn strip_id3(mut data: Vec<u8>) -> Vec<u8> {
    if data.len() > 10 && &data[0..3] == b"ID3" {
        let sz = data[6..10].iter().fold(0usize, |a, b| (a << 7) | (*b as usize & 0x7f));
        let footer = if data[5] & 0x10 != 0 { 10 } else { 0 };
        let skip = 10 + sz + footer;
        if skip < data.len() {
            data.drain(..skip);
        }
    }
    if data.len() > 128 && &data[data.len() - 128..data.len() - 125] == b"TAG" {
        data.truncate(data.len() - 128);
    }
    data
}

/// Render cover bytes to a 160x160 baseline JPEG (quality 80).
pub fn render_cover(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use image::imageops::FilterType;
    let img = image::load_from_memory(bytes).map_err(|e| e.to_string())?;
    let rgb = img.resize_to_fill(COVER_SIDE, COVER_SIDE, FilterType::Triangle).to_rgb8();
    let mut buf = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 80)
        .encode(rgb.as_raw(), COVER_SIDE, COVER_SIDE, image::ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}

fn cover_bytes(t: &ScanTrack) -> Option<Vec<u8>> {
    use lofty::picture::PictureType;
    use lofty::prelude::*;
    match &t.cover_src {
        CoverSrc::None => None,
        CoverSrc::File(p) => std::fs::read(p).ok(),
        CoverSrc::Embedded => {
            let tf = lofty::read_from_path(&t.path).ok()?;
            for tag in tf.tags() {
                let pic = tag.get_picture_type(PictureType::CoverFront).or_else(|| tag.pictures().first());
                if let Some(p) = pic {
                    return Some(p.data().to_vec());
                }
            }
            None
        }
    }
}

/// Write `data` to `dest` through a `.part` file so a cut-off write never
/// looks complete.
fn write_file(dest: &Path, data: &[u8]) -> Result<(), String> {
    let part = dest.with_extension("part");
    std::fs::write(&part, data).map_err(|e| format!("{}: {e}", dest.display()))?;
    std::fs::rename(&part, dest).map_err(|e| format!("{}: {e}", dest.display()))
}

fn copy_file(src: &Path, dest: &Path) -> Result<(), String> {
    let part = dest.with_extension("part");
    std::fs::copy(src, &part).map_err(|e| format!("{}: {e}", dest.display()))?;
    std::fs::rename(&part, dest).map_err(|e| format!("{}: {e}", dest.display()))
}

fn file_len(p: &Path) -> Option<u64> {
    std::fs::metadata(p).ok().filter(|m| m.is_file()).map(|m| m.len())
}

// ---------------------------------------------------------------------------
// Sync
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
pub struct Progress {
    /// "encode" | "copy" | "delete" | "index"
    pub phase: String,
    pub done: usize,
    pub total: usize,
    pub name: String,
}

#[derive(Serialize)]
pub struct SyncOut {
    pub tracks: usize,
    pub encoded: usize,
    pub copied: usize,
    pub deleted: usize,
    pub bytes_written: u64,
    pub library_bytes: u64,
    pub free: u64,
    /// Tracks that failed to encode (titles), left out of the library.
    pub failed: Vec<String>,
}

/// Encode the selected tracks into `cache` (reusing earlier encodes), then
/// make the watch library match: write new tracks and covers, delete tracks no
/// longer selected, and write index.json atomically. `free_space` returns the
/// free bytes on the watch drive.
pub fn sync(
    root: &Path,
    cache: &Path,
    ids: &[String],
    free_space: &dyn Fn() -> u64,
    progress: &(dyn Fn(Progress) + Sync),
) -> Result<SyncOut, String> {
    cancel_flag().store(false, Ordering::SeqCst);
    let cancel = cancel_flag();
    let (selected, playlists) = {
        let g = last_scan().lock().map_err(|e| e.to_string())?;
        let scan = g.as_ref().ok_or("scan_first")?;
        let want: HashSet<&str> = ids.iter().map(|s| s.as_str()).collect();
        let sel: Vec<ScanTrack> = scan.tracks.iter().filter(|t| want.contains(t.id.as_str())).cloned().collect();
        (sel, scan.playlists.clone())
    };
    let cache_tracks = cache.join("tracks");
    let cache_covers = cache.join("covers");
    std::fs::create_dir_all(&cache_tracks).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&cache_covers).map_err(|e| e.to_string())?;

    // 1. Encode into the local cache, in parallel.
    let todo: Vec<&ScanTrack> = selected
        .iter()
        .filter(|t| file_len(&cache_tracks.join(format!("{}.mp3", t.id))).unwrap_or(0) == 0)
        .collect();
    let total = todo.len();
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let failed: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
    let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2).clamp(1, 8);
    std::thread::scope(|s| {
        for _ in 0..workers.min(total.max(1)) {
            s.spawn(|| loop {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                let i = next.fetch_add(1, Ordering::SeqCst);
                let Some(t) = todo.get(i) else { break };
                let dest = cache_tracks.join(format!("{}.mp3", t.id));
                let res = if t.copy_as_is {
                    std::fs::read(&t.path).map_err(|e| e.to_string()).map(strip_id3)
                } else {
                    transcode(&t.path, cancel)
                };
                match res.and_then(|data| write_file(&dest, &data)) {
                    Ok(()) => {}
                    Err(e) => failed.lock().expect("lock").push((t.id.clone(), format!("{}: {e}", t.title))),
                }
                let n = done.fetch_add(1, Ordering::SeqCst) + 1;
                progress(Progress { phase: "encode".into(), done: n, total, name: t.title.clone() });
            });
        }
    });
    if cancel.load(Ordering::SeqCst) {
        return Err("cancelled".into());
    }
    let failed = failed.into_inner().map_err(|e| e.to_string())?;
    let failed_ids: HashSet<&str> = failed.iter().map(|(id, _)| id.as_str()).collect();
    let mut selected: Vec<ScanTrack> = selected.into_iter().filter(|t| !failed_ids.contains(t.id.as_str())).collect();

    // Covers, once per distinct image.
    let mut cover_ok: HashMap<String, bool> = HashMap::new();
    for t in &selected {
        if t.cover.is_empty() || cover_ok.contains_key(&t.cover) {
            continue;
        }
        let dest = cache_covers.join(format!("{}.jpg", t.cover));
        let ok = file_len(&dest).unwrap_or(0) > 0
            || cover_bytes(t)
                .and_then(|b| render_cover(&b).ok())
                .is_some_and(|jpg| write_file(&dest, &jpg).is_ok());
        cover_ok.insert(t.cover.clone(), ok);
    }
    for t in selected.iter_mut() {
        if !t.cover.is_empty() && cover_ok.get(&t.cover) != Some(&true) {
            t.cover.clear();
        }
    }

    // 2. Plan the watch changes.
    let mdir = music_dir(root);
    let wtracks = mdir.join("tracks");
    let wcovers = mdir.join("covers");
    std::fs::create_dir_all(&wtracks).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&wcovers).map_err(|e| e.to_string())?;

    let keep_tracks: HashSet<String> = selected.iter().map(|t| format!("{}.mp3", t.id)).collect();
    let keep_covers: HashSet<String> = selected
        .iter()
        .filter(|t| !t.cover.is_empty())
        .map(|t| format!("{}.jpg", t.cover))
        .collect();
    let mut removals: Vec<(PathBuf, u64)> = Vec::new();
    for (dir, keep) in [(&wtracks, &keep_tracks), (&wcovers, &keep_covers)] {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if !keep.contains(&name) {
                    let len = e.metadata().map(|m| m.len()).unwrap_or(0);
                    removals.push((e.path(), len));
                }
            }
        }
    }
    // (source in cache, destination on the watch, size to write, size replaced)
    let mut writes: Vec<(PathBuf, PathBuf, u64, u64, String)> = Vec::new();
    for t in &selected {
        let src = cache_tracks.join(format!("{}.mp3", t.id));
        let dst = wtracks.join(format!("{}.mp3", t.id));
        let len = file_len(&src).unwrap_or(0);
        let cur = file_len(&dst);
        if cur != Some(len) {
            writes.push((src, dst, len, cur.unwrap_or(0), t.title.clone()));
        }
    }
    for name in &keep_covers {
        let src = cache_covers.join(name);
        let dst = wcovers.join(name);
        let len = file_len(&src).unwrap_or(0);
        let cur = file_len(&dst);
        if cur != Some(len) {
            writes.push((src, dst, len, cur.unwrap_or(0), name.clone()));
        }
    }
    let needed: u64 = writes.iter().map(|w| w.2).sum::<u64>() + 64 * 1024;
    let freed: u64 = removals.iter().map(|r| r.1).sum::<u64>() + writes.iter().map(|w| w.3).sum::<u64>();
    let avail = free_space().saturating_add(freed);
    if needed + MARGIN > avail {
        return Err(format!("no_space:{needed}:{}", avail.saturating_sub(MARGIN)));
    }

    // 3. Apply. First an index of what stays, so the watch never points at a
    // deleted file; then deletions, copies, and the final index.
    let make_index = |only_present: bool| {
        let tracks: Vec<IndexTrack> = selected
            .iter()
            .filter(|t| !only_present || file_len(&wtracks.join(format!("{}.mp3", t.id))).is_some_and(|n| {
                n == file_len(&cache_tracks.join(format!("{}.mp3", t.id))).unwrap_or(0)
            }))
            .map(|t| IndexTrack {
                id: t.id.clone(),
                file: format!("tracks/{}.mp3", t.id),
                title: t.title.clone(),
                artist: t.artist.clone(),
                album: t.album.clone(),
                folder: t.folder.clone(),
                track: t.track,
                duration: t.duration,
                cover: if t.cover.is_empty() { String::new() } else { format!("covers/{}.jpg", t.cover) },
                size: file_len(&cache_tracks.join(format!("{}.mp3", t.id))).unwrap_or(0),
            })
            .collect();
        build_index(tracks, &playlists, &now_iso())
    };
    progress(Progress { phase: "index".into(), done: 0, total: 1, name: String::new() });
    write_index(&mdir, &make_index(true))?;

    let nrem = removals.len();
    for (i, (p, _)) in removals.iter().enumerate() {
        std::fs::remove_file(p).ok();
        progress(Progress { phase: "delete".into(), done: i + 1, total: nrem, name: String::new() });
    }
    let nwr = writes.len();
    let mut bytes_written = 0;
    for (i, (src, dst, len, _, name)) in writes.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            write_index(&mdir, &make_index(true))?;
            return Err("cancelled".into());
        }
        progress(Progress { phase: "copy".into(), done: i, total: nwr, name: name.clone() });
        if let Err(e) = copy_file(src, dst) {
            write_index(&mdir, &make_index(true))?;
            return Err(e);
        }
        bytes_written += len;
    }
    progress(Progress { phase: "copy".into(), done: nwr, total: nwr, name: String::new() });
    let index = make_index(true);
    write_index(&mdir, &index)?;

    Ok(SyncOut {
        tracks: index.tracks.len(),
        encoded: total - failed.len(),
        copied: writes.len(),
        deleted: nrem,
        bytes_written,
        library_bytes: dir_size(&mdir),
        free: free_space(),
        failed: failed.into_iter().map(|(_, e)| e).collect(),
    })
}

pub fn dir_size(p: &Path) -> u64 {
    let mut total = 0;
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let path = e.path();
            if path.is_dir() {
                total += dir_size(&path);
            } else if let Ok(m) = e.metadata() {
                total += m.len();
            }
        }
    }
    total
}

/// Group sizes by folder (the UI does the same; kept for tests).
#[cfg(test)]
pub fn folder_totals(tracks: &[TrackOut]) -> std::collections::BTreeMap<String, u64> {
    let mut m = std::collections::BTreeMap::new();
    for t in tracks {
        *m.entry(t.folder.clone()).or_insert(0) += t.est_size;
    }
    m
}

// ---------------------------------------------------------------------------
// Recycle bin on the watch drive
// ---------------------------------------------------------------------------

pub fn recycle_dir(root: &Path) -> PathBuf {
    root.join("$RECYCLE.BIN")
}

/// Delete everything inside `$RECYCLE.BIN` on the watch drive. Returns the
/// bytes freed. Windows recreates the folder when needed.
pub fn empty_recycle(root: &Path) -> u64 {
    let dir = recycle_dir(root);
    let before = dir_size(&dir);
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                std::fs::remove_dir_all(&p).ok();
            } else {
                std::fs::remove_file(&p).ok();
            }
        }
    }
    before.saturating_sub(dir_size(&dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_is_stable_and_case_insensitive() {
        let a = track_id("Rock/Queen/01 Song.flac", 1234, 1700000000);
        assert_eq!(a.len(), 8);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(a, track_id("rock\\queen\\01 song.flac", 1234, 1700000000));
        assert_ne!(a, track_id("Rock/Queen/01 Song.flac", 1235, 1700000000));
        assert_ne!(a, track_id("Rock/Queen/01 Song.flac", 1234, 1700000001));
        assert_eq!(cover_id(b"abc"), cover_id(b"abc"));
        assert_ne!(cover_id(b"abc"), cover_id(b"abd"));
    }

    #[test]
    fn folder_relativization() {
        let root = Path::new("/lib");
        let (rel, folder) = rel_and_folder(root, Path::new("/lib/Rock/Queen/a.mp3")).unwrap();
        assert_eq!(rel, "Rock/Queen/a.mp3");
        assert_eq!(folder, "Rock/Queen");
        let (rel, folder) = rel_and_folder(root, Path::new("/lib/a.mp3")).unwrap();
        assert_eq!(rel, "a.mp3");
        assert_eq!(folder, "");
        assert!(rel_and_folder(root, Path::new("/other/a.mp3")).is_none());
    }

    #[test]
    fn size_estimate() {
        assert_eq!(estimate_size(0.0), 0);
        assert_eq!(estimate_size(215.4), 1_723_200);
        assert_eq!(estimate_size(-3.0), 0);
        let t = |folder: &str, est| TrackOut {
            id: String::new(),
            rel: String::new(),
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            folder: folder.into(),
            track: 0,
            duration: 0.0,
            est_size: est,
            cover: String::new(),
        };
        let m = folder_totals(&[t("A", 10), t("A", 5), t("", 1)]);
        assert_eq!(m["A"], 15);
        assert_eq!(m[""], 1);
    }

    #[test]
    fn index_shape() {
        let tr = |id: &str, folder: &str, n: u32| IndexTrack {
            id: id.into(),
            file: format!("tracks/{id}.mp3"),
            title: format!("T{n}"),
            artist: String::new(),
            album: String::new(),
            folder: folder.into(),
            track: n,
            duration: 215.4,
            cover: String::new(),
            size: 1720000,
        };
        let pls = [
            Playlist { name: "Mix".into(), tracks: vec!["bbbbbbbb".into(), "zzzzzzzz".into()] },
            Playlist { name: "Gone".into(), tracks: vec!["zzzzzzzz".into()] },
        ];
        let idx = build_index(
            vec![tr("bbbbbbbb", "Rock/Queen", 3), tr("aaaaaaaa", "Rock/Queen", 1), tr("cccccccc", "", 0)],
            &pls,
            "2026-11-13T12:00:00Z",
        );
        let v = serde_json::to_value(&idx).unwrap();
        assert_eq!(v["version"], 1);
        assert_eq!(v["generated"], "2026-11-13T12:00:00Z");
        let ids: Vec<&str> = v["tracks"].as_array().unwrap().iter().map(|t| t["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["cccccccc", "aaaaaaaa", "bbbbbbbb"]);
        let t0 = &v["tracks"][1];
        for k in ["id", "file", "title", "artist", "album", "folder", "track", "duration", "cover", "size"] {
            assert!(t0.get(k).is_some(), "missing {k}");
        }
        assert_eq!(t0["file"], "tracks/aaaaaaaa.mp3");
        assert_eq!(t0["track"], 1);
        assert_eq!(v["playlists"].as_array().unwrap().len(), 1);
        assert_eq!(v["playlists"][0]["tracks"], serde_json::json!(["bbbbbbbb"]));
    }

    #[test]
    fn m3u_mapping() {
        let mut lookup = HashMap::new();
        lookup.insert("rock/queen/01 a.mp3".to_string(), "11111111".to_string());
        lookup.insert("pop/b.flac".to_string(), "22222222".to_string());
        lookup.insert("c.wav".to_string(), "33333333".to_string());
        let root = Path::new("C:\\Music");
        let text = "\u{feff}#EXTM3U\n#EXTINF:123,Queen - A\nQueen\\01 A.mp3\n../Pop/b.flac\nC:\\Music\\c.wav\nmissing.mp3\nD:\\Other\\x.mp3\n";
        let ids = parse_m3u(text, "Rock", root, &lookup);
        assert_eq!(ids, ["11111111", "22222222", "33333333"]);
    }

    #[test]
    fn hebrew_mojibake_is_repaired() {
        assert_eq!(fix_hebrew("éùøàì åøãéâø"), "ישראל ורדיגר");
        assert_eq!(fix_hebrew("Queen"), "Queen");
        assert_eq!(fix_hebrew("Café del Mar"), "Café del Mar");
        assert_eq!(fix_hebrew("ישראל"), "ישראל");
    }

    #[test]
    fn id3_is_stripped() {
        let mut d = b"ID3\x04\x00\x00\x00\x00\x00\x05hello".to_vec();
        d.extend_from_slice(&[0xFF, 0xFB, 0x50, 0x00]);
        assert_eq!(strip_id3(d), vec![0xFF, 0xFB, 0x50, 0x00]);
    }

    /// Parse MP3 frame headers; returns (frames, bitrates seen, sample rates seen).
    fn mp3_frames(data: &[u8]) -> (usize, HashSet<u32>, HashSet<u32>) {
        const BR: [u32; 16] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
        const SR: [u32; 4] = [44100, 48000, 32000, 0];
        let (mut i, mut n) = (0, 0);
        let (mut brs, mut srs) = (HashSet::new(), HashSet::new());
        while i + 4 <= data.len() {
            let h = &data[i..i + 4];
            // MPEG-1 Layer III sync.
            if h[0] == 0xFF && (h[1] & 0xFE) == 0xFA {
                let br = BR[(h[2] >> 4) as usize];
                let sr = SR[((h[2] >> 2) & 3) as usize];
                if br == 0 || sr == 0 {
                    break;
                }
                let pad = ((h[2] >> 1) & 1) as usize;
                let len = (144 * br * 1000 / sr) as usize + pad;
                brs.insert(br);
                srs.insert(sr);
                n += 1;
                i += len;
            } else {
                break;
            }
        }
        (n, brs, srs)
    }

    #[test]
    fn encodes_tone_to_64k_mp3() {
        // 3 s of a 440 Hz tone at 48 kHz: also exercises resampling to 44.1 kHz.
        let rate = 48000;
        let n = rate as usize * 3;
        let l: Vec<f32> = (0..n).map(|i| (i as f32 * 440.0 * std::f32::consts::TAU / rate as f32).sin() * 0.5).collect();
        let r = l.clone();
        let mp3 = encode_pcm(&l, &r, rate).unwrap();
        let (frames, brs, srs) = mp3_frames(&mp3);
        assert!(frames > 100, "frames {frames}");
        assert_eq!(brs, HashSet::from([64]));
        assert_eq!(srs, HashSet::from([44100]));
        // About 3 s at 8000 B/s.
        assert!((20_000..30_000).contains(&mp3.len()), "len {}", mp3.len());

        // And the output decodes back with symphonia.
        let dir = std::env::temp_dir().join(format!("dtg1_music_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("tone.mp3");
        std::fs::write(&f, &mp3).unwrap();
        let re = transcode(&f, &AtomicBool::new(false)).unwrap();
        assert!(mp3_frames(&re).0 > 100);

        // A mono 16-bit WAV at 22.05 kHz goes through the decoder path and
        // comes out as 44.1 kHz 64 kbps too.
        let sr: u32 = 22050;
        let pcm: Vec<i16> = (0..sr * 2)
            .map(|i| ((i as f32 * 330.0 * std::f32::consts::TAU / sr as f32).sin() * 12000.0) as i16)
            .collect();
        let data_len = pcm.len() as u32 * 2;
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_len).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&sr.to_le_bytes());
        wav.extend_from_slice(&(sr * 2).to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_len.to_le_bytes());
        for s in &pcm {
            wav.extend_from_slice(&s.to_le_bytes());
        }
        let w = dir.join("tone.wav");
        std::fs::write(&w, &wav).unwrap();
        let from_wav = transcode(&w, &AtomicBool::new(false)).unwrap();
        let (frames, brs, srs) = mp3_frames(&from_wav);
        assert!(frames > 60, "frames {frames}");
        assert_eq!(brs, HashSet::from([64]));
        assert_eq!(srs, HashSet::from([44100]));
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[cfg(test)]
mod itest {
    //! Real-library check: `MUSIC_TEST_DIR=<folder> cargo test -p dt-g1-studio-app --lib music::itest -- --ignored --nocapture`
    use super::*;

    #[test]
    #[ignore]
    fn scan_and_sync_real_folder() {
        let Ok(dir) = std::env::var("MUSIC_TEST_DIR") else { return };
        let t0 = std::time::Instant::now();
        let s = scan(Path::new(&dir), &|_, _| {}).unwrap();
        println!("scanned {} tracks ({} skipped) in {:?}", s.tracks.len(), s.skipped, t0.elapsed());
        let pick: Vec<TrackOut> = s.tracks.iter().take(4).cloned().collect();
        for t in &pick {
            println!("  {} | {} | {} | {:.1}s est {} cover {}", t.folder, t.title, t.artist, t.duration, t.est_size, t.cover);
        }
        if let Some(long) = pick.iter().max_by(|a, b| a.duration.total_cmp(&b.duration)) {
            let p = Path::new(&dir).join(&long.rel);
            let t = std::time::Instant::now();
            let mp3 = transcode(&p, &AtomicBool::new(false)).unwrap();
            println!("single transcode: {:.0}s of audio in {:?} ({} bytes)", long.duration, t.elapsed(), mp3.len());
        }
        let tmp = std::env::temp_dir().join(format!("dtg1_music_it_{}", std::process::id()));
        let (root, cache) = (tmp.join("watch"), tmp.join("cache"));
        let ids: Vec<String> = pick.iter().map(|t| t.id.clone()).collect();
        let t1 = std::time::Instant::now();
        let out = sync(&root, &cache, &ids, &|| 290 << 20, &|_| {}).unwrap();
        let audio: f64 = pick.iter().map(|t| t.duration).sum();
        println!(
            "synced {} tracks ({:.0}s audio) in {:?}: written {} failed {:?}",
            out.tracks, audio, t1.elapsed(), out.bytes_written, out.failed
        );
        let idx = read_index(&root);
        assert_eq!(idx.tracks.len(), out.tracks);
        for t in &idx.tracks {
            println!("  {} {} size {} est {}", t.id, t.title, t.size, estimate_size(t.duration));
        }
        // Second sync reuses the cache and writes nothing.
        let out2 = sync(&root, &cache, &ids[..2], &|| 290 << 20, &|_| {}).unwrap();
        assert_eq!(out2.encoded, 0);
        assert_eq!(out2.copied, 0);
        assert_eq!(read_index(&root).tracks.len(), 2);
        std::fs::remove_dir_all(&tmp).ok();
    }
}