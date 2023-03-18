pub(crate) use libvvenc_sys as ffi;

mod error;
pub(crate) use error::FFIStatusToResult;

mod chroma_format;
mod config;
mod encoder;
mod encoder_output;
mod preset;
mod slice_type;
mod yuv_buffer;

pub use chroma_format::ChromaFormat;
pub use config::Config;
pub use encoder::Encoder;
pub use encoder_output::{EncoderOutput, EncoderOutputData};
pub use error::Error;
pub use preset::Preset;
pub use slice_type::SliceType;
pub use yuv_buffer::{YUVBuffer, YUVPlane};

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use super::*;

    #[test]
    fn basic() {
        let width = 1280;
        let height = 720;
        let framerate = 30;
        let bitrate = 2_000_000;
        let qp = 32;
        let preset = Preset::Medium;

        let config = Config::new(width, height, framerate, bitrate, qp, preset).unwrap();
        let mut encoder = Encoder::with_config(config).unwrap();

        let mut y_plane = Vec::with_capacity((width * height) as usize);
        let mut u_plane = Vec::with_capacity((width * height) as usize >> 1);
        let mut v_plane = Vec::with_capacity((width * height) as usize >> 1);

        // A green frame
        y_plane.resize((width * height) as usize, 0);
        u_plane.resize((width * height) as usize >> 1, 0);
        v_plane.resize((width * height) as usize >> 1, 0);

        let yuv_buffer = YUVBuffer {
            planes: [
                YUVPlane {
                    buffer: &y_plane,
                    width,
                    height,
                    stride: width,
                },
                YUVPlane {
                    buffer: &u_plane,
                    width: width >> 1,
                    height: height >> 1,
                    stride: width >> 1,
                },
                YUVPlane {
                    buffer: &v_plane,
                    width: width >> 1,
                    height: height >> 1,
                    stride: width >> 1,
                },
            ],
            sequence_number: 0,
            composition_timestamp: Some(0),
        };

        let mut output_file = File::create("test.vvc").unwrap();

        let mut encode = |buffer| match encoder.encode(buffer).unwrap() {
            EncoderOutput::None => {
                println!("No output yet");
                false
            }
            EncoderOutput::Data(data, done) => {
                println!("got output!");
                assert_eq!(output_file.write(data.payload).unwrap(), data.payload.len());
                done
            }
        };

        let mut encoding_done = encode(Some(&yuv_buffer));
        while !encoding_done {
            encoding_done = encode(None);
        }

        println!("encoding done!");
    }
}
