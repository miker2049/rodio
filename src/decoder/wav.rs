use std::io::{Read, Seek, SeekFrom};
use std::cmp::Ordering;
use super::Decoder;
use super::conversions;

use cpal::{self, Endpoint, Voice};
use hound::WavIntoSamples;
use hound::WavReader;
use hound::WavSpec;

pub struct WavDecoder {
    reader: Box<Iterator<Item=i16> + Send>,
    voice: Voice,
}

impl WavDecoder {
    pub fn new<R>(endpoint: &Endpoint, mut data: R) -> Result<WavDecoder, R>
                  where R: Read + Seek + Send + 'static
    {
        if !is_wave(data.by_ref()) {
            return Err(data);
        }

        let reader = WavReader::new(data).unwrap();
        let spec = reader.spec();

        // choosing a format amongst the ones available
        let voice_format = {
            let mut formats_list = endpoint.get_supported_formats_list().unwrap().collect::<Vec<_>>();
            formats_list.sort_by(|f1, f2| {
                if (f1.samples_rate.0 % spec.sample_rate) == 0 && (f2.samples_rate.0 % spec.sample_rate) != 0 {
                    return Ordering::Greater;
                } else if (f2.samples_rate.0 % spec.sample_rate) == 0 && (f1.samples_rate.0 % spec.sample_rate) != 0 {
                    return Ordering::Less;
                }

                if f1.channels.len() >= spec.channels as usize && f1.channels.len() < f2.channels.len() {
                    return Ordering::Greater;
                } else if f2.channels.len() >= spec.channels as usize && f2.channels.len() < f1.channels.len() {
                    return Ordering::Less;
                }

                if f1.data_type == cpal::SampleFormat::I16 && f2.data_type != cpal::SampleFormat::I16 {
                    return Ordering::Greater;
                } else if f2.data_type == cpal::SampleFormat::I16 && f1.data_type != cpal::SampleFormat::I16 {
                    return Ordering::Less;
                }

                Ordering::Equal
            });
            formats_list.into_iter().next().unwrap()
        };

        let voice = Voice::new(endpoint, &voice_format).unwrap();

        let reader = reader.into_samples().map(|s| s.unwrap_or(0));
        let reader = conversions::ChannelsCountConverter::new(reader, spec.channels,
                                                              voice.get_channels());
        let reader = conversions::SamplesRateConverter::new(reader, cpal::SamplesRate(spec.sample_rate),
                                                            voice.get_samples_rate());

        Ok(WavDecoder {
            reader: Box::new(reader),
            voice: voice,
        })
    }
}

/// Returns true if the stream contains WAV data, then resets it to where it was.
fn is_wave<R>(mut data: R) -> bool where R: Read + Seek {
    let stream_pos = data.seek(SeekFrom::Current(0)).unwrap();

    if WavReader::new(data.by_ref()).is_err() {
        data.seek(SeekFrom::Start(stream_pos)).unwrap();
        return false;
    }

    data.seek(SeekFrom::Start(stream_pos)).unwrap();
    true
}

impl Decoder for WavDecoder {
    fn write(&mut self) {
        let (min, _) = self.reader.size_hint();

        if min == 0 {
            // finished
            return;
        }

        {
            let mut buffer = self.voice.append_data(min);
            conversions::convert_and_write(self.reader.by_ref(), &mut buffer);
        }

        self.voice.play();
    }
}
