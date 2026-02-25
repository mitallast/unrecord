use crate::audio::{
    CoreAudioUnit, estimate_latency_by_peak_in_window_f32_stereo_interleaved_frames,
    make_impulse_test_f32_stereo_interleaved, read_file, sample_stats, write_file,
};
use anyhow::{Context, Result};
use async_std::task;
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

pub struct RecordSession {
    destination_path: PathBuf,
    io_unit: Option<AudioUnit>,
    device_id: AudioObjectID,
    sample_rate: f64,
    source_samples: Vec<f32>,
    pre_silence_frames: usize,
    post_silence_frames: usize,
    record_duration: Duration,
    test_samples: Vec<f32>,
    output_samples: Arc<Mutex<VecDeque<f32>>>,
    input_samples: Arc<Mutex<VecDeque<f32>>>,
    output_len: usize,
}

impl RecordSession {
    fn new_blocking<F: AsRef<Path>, T: AsRef<Path>>(
        device_id: AudioObjectID,
        source_path: F,
        destination_path: T,
    ) -> Result<Self> {
        let source_path = source_path.as_ref().to_path_buf();
        let destination_path = destination_path.as_ref().to_path_buf();

        let (spec, source_samples) = read_file(&source_path)?;
        let pre_silence_frames = (0.25 * spec.sample_rate as f64) as usize;
        let post_silence_frames = (0.75 * spec.sample_rate as f64) as usize;

        let impulse_amp: f32 = 0.75;
        let test_samples = make_impulse_test_f32_stereo_interleaved(
            pre_silence_frames,
            post_silence_frames,
            impulse_amp,
            impulse_amp,
        );

        let output_len = test_samples.len() + source_samples.len();
        let record_duration = output_len as f64 / spec.sample_rate as f64 / 2.0 + 1.0;
        let record_duration = Duration::from_secs_f64(record_duration);

        let mut output_samples: VecDeque<f32> = VecDeque::with_capacity(output_len);
        output_samples.extend(test_samples.iter().cloned());
        output_samples.extend(source_samples.iter().cloned());
        let output_samples = Arc::new(Mutex::new(output_samples));
        let input_samples = Arc::new(Mutex::new(VecDeque::with_capacity(output_len)));

        let io_unit = AudioUnit::new(IOType::HalOutput)?;
        let mut session = Self {
            destination_path,
            io_unit: Some(io_unit),
            device_id,
            sample_rate: spec.sample_rate as f64,
            source_samples,
            pre_silence_frames,
            post_silence_frames,
            record_duration,
            test_samples,
            output_samples,
            input_samples,
            output_len,
        };
        session.init()?;
        Ok(session)
    }

    fn init(&mut self) -> Result<()> {
        let io_unit = self.io_unit.as_mut().unwrap();
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

        let output_consumer = self.output_samples.clone();
        let input_producer = self.input_samples.clone();

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

        io_unit.set_input_callback(move |args| {
            let num_frames = args.num_frames;
            let data: Interleaved<f32> = args.data;
            let mut buffer = input_producer.lock().unwrap();
            for s in 0..(num_frames * 2) {
                buffer.push_back(data.buffer[s]);
            }
            Ok(())
        })?;

        Ok(())
    }

    fn start_blocking(&mut self) -> Result<()> {
        self.io_unit.as_mut().unwrap().start()?;
        Ok(())
    }

    fn stop_blocking(&mut self) -> Result<()> {
        let mut io_unit = self.io_unit.take().context("No io_unit")?;
        io_unit.stop()?;
        drop(io_unit);

        let input_samples: Vec<f32> = self.input_samples.lock().unwrap().clone().into();
        info!("input   = {} samples", input_samples.len());
        if input_samples.len() < self.output_len {
            return Err(anyhow::anyhow!("Not enough samples"));
        }

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

        info!("write result to {:?}", &self.destination_path);
        write_file(&self.destination_path, self.sample_rate, &final_samples)?;
        info!("done");
        Ok(())
    }

    pub async fn new<F: AsRef<Path>, T: AsRef<Path>>(
        device_id: AudioObjectID,
        source_path: F,
        destination_path: T,
    ) -> Result<Self> {
        task::block_on(async move { Self::new_blocking(device_id, source_path, destination_path) })
    }

    pub async fn start(&mut self) -> Result<()> {
        task::block_on(async { self.start_blocking() })
    }

    pub async fn stop(&mut self) -> Result<()> {
        task::block_on(async { self.stop_blocking() })
    }

    pub async fn wait(&mut self) {
        async_std::task::sleep(self.record_duration).await;
    }
}
