use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};

use crate::apu::APU;

fn make_stream<T: cpal::SizedSample + cpal::FromSample<f32>>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut consumer: ringbuf::HeapCons<f32>,
) -> cpal::Stream
{
    let stream = device.build_output_stream(
        *config, 
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for sample in data.iter_mut() {
                let raw_sample = consumer.try_pop().unwrap_or(0.);
                
                *sample = T::from_sample(raw_sample);
            }
        }, 
        |err| eprintln!("Audio error: {}", err), 
        None
    ).unwrap();

    stream
}

pub struct AudioPlayer {
    pub cycles_per_sample:  f32,
    pub accumulator:        f32,

    pub num_channels:       usize,
    pub volume:             f32,

    pub producer:           ringbuf::HeapProd<f32>,
    pub stream:             cpal::Stream,
}

impl AudioPlayer {
    pub fn new() -> AudioPlayer {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No default output device.");
        
        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate() as f32;
        let channels = config.channels() as usize;

        let buf = ringbuf::HeapRb::<f32>::new(10000);
        let (producer, consumer) = buf.split();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into(), consumer),
            cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into(), consumer),
            cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into(), consumer),
            _ => panic!("Unsupported sample format"),
        };

        AudioPlayer
        {
            cycles_per_sample:  1789773.0 / sample_rate,
            accumulator:        0.,

            num_channels:       channels,
            volume:             0.5,

            stream,
            producer
        }
    }

    pub fn tick(&mut self, apu: &APU) {
        self.accumulator += 1.;

        if self.accumulator >= self.cycles_per_sample {
            self.accumulator -= self.cycles_per_sample;

            let sample = apu.output();
            
            for _ in 0..self.num_channels {
                let _ = self.producer.try_push(sample * self.volume);
            }
        }
    }

    pub fn play(&mut self) {
        self.stream.play().unwrap();
    }

    pub fn pause(&mut self) {
        self.stream.pause().unwrap();
    }
}