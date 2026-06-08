use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Recorder {
    stream: cpal::Stream,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
}

// cpal::Stream ist auf manchen Plattformen nicht Send. Wir greifen nur vom erzeugenden
// Thread darauf zu (Start/Stop). Der Recorder selbst wird im Zustand gehalten.
unsafe impl Send for Recorder {}
unsafe impl Sync for Recorder {}

impl Recorder {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "Kein Mikrofon-Geraet gefunden.".to_string())?;

        let config = device
            .default_input_config()
            .map_err(|e| format!("Mikrofon-Konfiguration fehlgeschlagen: {}", e))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let samples_clone = samples.clone();

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if let Ok(mut buf) = samples_clone.lock() {
                        buf.extend_from_slice(data);
                    }
                },
                |err| eprintln!("[Blixt Audio] Stream-Fehler: {}", err),
                None,
            )
            .map_err(|e| format!("Audiostream konnte nicht geoeffnet werden: {}", e))?;

        Ok(Recorder {
            stream,
            samples,
            sample_rate,
            channels,
        })
    }

    pub fn start(&self) -> Result<(), String> {
        self.stream
            .play()
            .map_err(|e| format!("Aufnahme konnte nicht gestartet werden: {}", e))
    }

    /// Stoppt die Aufnahme, kodiert nach WAV (16 bit mono), schreibt Temp-Datei, gibt Pfad zurueck.
    pub fn stop_and_save(self) -> Result<PathBuf, String> {
        drop(self.stream);

        let samples = self
            .samples
            .lock()
            .map_err(|_| "Interner Fehler beim Lesen der Aufnahme.".to_string())?;

        if samples.is_empty() {
            return Err("Keine Aufnahme erkannt.".to_string());
        }

        // Mehrkanal -> Mono mischen
        let mono: Vec<f32> = if self.channels > 1 {
            let ch = self.channels as usize;
            samples
                .chunks(ch)
                .map(|chunk| chunk.iter().sum::<f32>() / ch as f32)
                .collect()
        } else {
            samples.clone()
        };

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let path = std::env::temp_dir().join(format!("blixt_{}.wav", ts));

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(&path, spec)
            .map_err(|e| format!("WAV-Datei konnte nicht erstellt werden: {}", e))?;

        for &s in &mono {
            let sample = (s.max(-1.0).min(1.0) * i16::MAX as f32) as i16;
            writer
                .write_sample(sample)
                .map_err(|e| format!("WAV-Schreibfehler: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("WAV-Finalisierung fehlgeschlagen: {}", e))?;

        Ok(path)
    }

    pub fn recording_duration_seconds(&self) -> f64 {
        if let Ok(buf) = self.samples.lock() {
            buf.len() as f64 / (self.sample_rate as f64 * self.channels as f64)
        } else {
            0.0
        }
    }
}
