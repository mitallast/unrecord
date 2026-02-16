use crate::audio::{
    CoreAudioUnit, estimate_latency_by_peak_in_window_f32_stereo_interleaved_frames,
    make_impulse_test_f32_stereo_interleaved, sample_stats,
};

use crate::wav_file::{read_file, write_file};

use anyhow::{Context, Result};
use coreaudio::audio_unit::audio_format::LinearPcmFlags;
use coreaudio::audio_unit::render_callback::data::Interleaved;
use coreaudio::audio_unit::{
    AudioUnit, Element, IOType, SampleFormat, Scope, StreamFormat, render_callback,
};
use log::info;
use objc2_audio_toolbox::kAudioUnitProperty_Latency;
use objc2_core_audio::{AudioObjectID, kAudioDevicePropertySafetyOffset};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const SAMPLE_FORMAT: SampleFormat = SampleFormat::F32;

type Args = render_callback::Args<Interleaved<f32>>;

pub struct RecordTask {
    device_id: AudioObjectID,
    source_path: PathBuf,
    destination_path: PathBuf,

    sample_rate: f64,
    source_samples: Vec<f32>,
    pre_silence_frames: usize,
    post_silence_frames: usize,
    output_len: usize,
    record_duration: Duration,
    test_samples: Vec<f32>,
    input_samples: Arc<Mutex<VecDeque<f32>>>,
}

impl RecordTask {
    pub fn new<F: AsRef<Path>, T: AsRef<Path>>(
        device_id: AudioObjectID,
        source_path: F,
        destination_path: T,
    ) -> Result<Self> {
        let from_path = source_path.as_ref().to_path_buf();
        let to_path = destination_path.as_ref().to_path_buf();

        let (sample_rate, source_samples) = read_file(&from_path)?;

        let pre_silence_frames = (0.25 * sample_rate) as usize;
        let post_silence_frames = (0.75 * sample_rate) as usize;

        let impulse_amp: f32 = 0.75;
        let test_samples = make_impulse_test_f32_stereo_interleaved(
            pre_silence_frames,
            post_silence_frames,
            impulse_amp,
            impulse_amp,
        );

        let output_len = test_samples.len() + source_samples.len();
        let record_duration = output_len as f64 / sample_rate / 2.0 + 1.0;
        let record_duration = Duration::from_secs_f64(record_duration);

        let input_samples = Arc::new(Mutex::new(VecDeque::<f32>::with_capacity(output_len)));

        Ok(Self {
            device_id,
            source_path: from_path,
            destination_path: to_path,
            sample_rate,
            source_samples,
            pre_silence_frames,
            post_silence_frames,
            output_len,
            record_duration,
            test_samples,
            input_samples,
        })
    }

    pub fn source_path(&self) -> String {
        self.source_path.to_str().unwrap().to_string()
    }
    pub fn destination_path(&self) -> String {
        self.destination_path.to_str().unwrap().to_string()
    }

    pub fn record(&self) -> Result<()> {
        let mut io_unit = AudioUnit::new(IOType::HalOutput)?;
        io_unit.enable_io_input()?;
        io_unit.enable_io_output()?;
        io_unit.set_device(self.device_id)?;

        io_unit.set_sample_rate(self.sample_rate)?;
        io_unit.set_io_buffer_size(16)?;

        let output_latency: u32 =
            io_unit.get_property(kAudioUnitProperty_Latency, Scope::Global, Element::Output)?;

        let output_safety_offset: u32 = io_unit.get_property(
            kAudioDevicePropertySafetyOffset,
            Scope::Output,
            Element::Output,
        )?;
        let input_safety_offset: u32 = io_unit.get_property(
            kAudioDevicePropertySafetyOffset,
            Scope::Input,
            Element::Input,
        )?;
        let io_sample_rate = io_unit.get_sample_rate()?;
        let io_buffer_size = io_unit.get_io_buffer_size()?;

        info!("(AU) output latency       : {}", output_latency);
        info!("(AU) sample rate          : {}", io_sample_rate);
        info!("(AU) buffer size          : {}", io_buffer_size);
        info!("(AU) output safety offset : {}", output_safety_offset);
        info!("(AU) input safety offset  : {}", input_safety_offset);

        let flags = LinearPcmFlags::IS_FLOAT | LinearPcmFlags::IS_PACKED;

        let in_stream_format = StreamFormat {
            sample_rate: self.sample_rate,
            sample_format: SAMPLE_FORMAT,
            flags,
            channels: 2,
        };

        let out_stream_format = StreamFormat {
            sample_rate: self.sample_rate,
            sample_format: SAMPLE_FORMAT,
            flags,
            channels: 2,
        };

        io_unit.set_input_stream_format_spec(&in_stream_format)?;
        io_unit.set_output_stream_format_spec(&out_stream_format)?;

        let mut output_samples: VecDeque<f32> = VecDeque::with_capacity(self.output_len);
        output_samples.extend(self.test_samples.iter().cloned());
        output_samples.extend(self.source_samples.iter().cloned());

        let output_samples = Arc::new(Mutex::new(output_samples));
        let output_consumer = output_samples.clone();
        io_unit.set_render_callback(move |args: Args| {
            let num_frames = args.num_frames;
            let data: Interleaved<f32> = args.data;
            let mut buffer = output_consumer.lock().unwrap();
            let zero: f32 = 0.0;
            for s in 0..(num_frames * 2) {
                data.buffer[s] = buffer.pop_front().unwrap_or(zero);
            }
            Ok(())
        })?;

        // write input to file
        let input_producer = self.input_samples.clone();

        io_unit.set_input_callback(move |args| {
            let num_frames = args.num_frames;
            let data: Interleaved<f32> = args.data;
            let mut buffer = input_producer.lock().unwrap();
            for s in 0..(num_frames * 2) {
                buffer.push_back(data.buffer[s]);
            }
            Ok(())
        })?;

        io_unit.start()?;
        std::thread::sleep(self.record_duration);
        io_unit.stop()?;
        drop(io_unit);

        let input_samples: Vec<f32> = self.input_samples.lock().unwrap().clone().into();
        info!("input   = {} samples", input_samples.len());

        let latency_frames = estimate_latency_by_peak_in_window_f32_stereo_interleaved_frames(
            &input_samples,
            self.pre_silence_frames,
            self.post_silence_frames,
        )
        .context("latency estimation error")?;

        let latency_samples = latency_frames * 2;
        let start_sample = (self.test_samples.len() as isize + latency_samples) as usize;
        let end_sample = start_sample + self.source_samples.len();

        info!("latency = {} frames", latency_frames);
        info!("latency = {} samples", latency_samples);
        info!("start   = {} sample", start_sample);
        info!("end     = {} sample", end_sample);

        let final_samples = input_samples[start_sample..end_sample].to_vec();

        sample_stats(&self.source_samples);
        sample_stats(&final_samples);

        write_file(&self.destination_path, self.sample_rate, &final_samples)?;

        Ok(())
    }
}
