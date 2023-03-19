use std::{fs::File, io::Write, path::PathBuf};

use anyhow::Result;
use clap::Parser;

/// Encode VVC bitstreams from Y4M inputs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input Y4M file path with yuv420p pixel format
    #[arg(short, long)]
    input: PathBuf,

    /// Output VVC file path
    #[arg(short, long)]
    output: PathBuf,
}

struct Logger;
impl vvenc::LoggingHandler for Logger {
    fn handle_message(&self, verbosity: vvenc::config::Verbosity, message: String) {
        let verbosity = format!("{:?}", verbosity).to_uppercase();
        println!("[{}] {}", verbosity, message);
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = File::open(args.input)?;
    let mut output = File::create(args.output)?;

    let mut y4m_decoder = y4m::Decoder::new(input)?;

    let input_colorspace = y4m_decoder.get_colorspace();
    if !matches!(input_colorspace, y4m::Colorspace::C420jpeg) {
        panic!(
            "Only yuv420p is currently supported. Input file has {:?}",
            input_colorspace
        );
    }

    let width = y4m_decoder.get_width() as i32;
    let height = y4m_decoder.get_height() as i32;

    let framerate = y4m_decoder.get_framerate();
    let framerate = (framerate.num / framerate.den) as i32;

    let bitrate = 2_000_000;
    let qp = 32;
    let preset = vvenc::config::Preset::Medium;

    let config = vvenc::ConfigBuilder::with_default(width, height, framerate, bitrate, qp, preset)?
        .with_verbosity(vvenc::config::Verbosity::Verbose)
        .with_logging_handler(Logger);
    let mut vvc_encoder = vvenc::Encoder::with_config(config)?;

    let mut input_frame_num = 0;
    let mut output_frame_num = 0;

    let mut y_buffer = Vec::with_capacity((width * height) as usize);
    let mut u_buffer = Vec::with_capacity(((width >> 1) * (height >> 1)) as usize);
    let mut v_buffer = Vec::with_capacity(((width >> 1) * (height >> 1)) as usize);
    while let Ok(frame) = y4m_decoder.read_frame() {
        println!("processing input frame {}", input_frame_num);

        // Need to change this conversion below when adding support to formats other than yuv420p in the input
        let map_to_yuv_buffer = |input_buffer: &[u8], yuv_buffer: &mut Vec<i16>| {
            yuv_buffer.clear();
            input_buffer
                .iter()
                // we left-shift 2 bits due to VVenC expecting 10 bits, vs 8 bits on the input.
                .for_each(|sample| yuv_buffer.push(i16::from(*sample) << 2));
        };

        map_to_yuv_buffer(frame.get_y_plane(), &mut y_buffer);
        map_to_yuv_buffer(frame.get_u_plane(), &mut u_buffer);
        map_to_yuv_buffer(frame.get_v_plane(), &mut v_buffer);

        let yuv_buffer = vvenc::YUVBuffer {
            planes: [
                vvenc::YUVPlane {
                    buffer: &y_buffer,
                    width,
                    height,
                    stride: width,
                },
                vvenc::YUVPlane {
                    buffer: &u_buffer,
                    width: width >> 1,
                    height: height >> 1,
                    stride: width >> 1,
                },
                vvenc::YUVPlane {
                    buffer: &v_buffer,
                    width: width >> 1,
                    height: height >> 1,
                    stride: width >> 1,
                },
            ],
            sequence_number: input_frame_num,
            composition_timestamp: Some(input_frame_num),
        };

        match vvc_encoder.encode(Some(&yuv_buffer))? {
            vvenc::EncoderOutput::None => {}
            vvenc::EncoderOutput::Data(data, _) => {
                println!("got output frame {}", output_frame_num);
                assert_eq!(output.write(data.payload)?, data.payload.len());
                output_frame_num += 1;
            }
        }

        input_frame_num += 1;
    }

    let mut encoding_done = false;
    while !encoding_done {
        match vvc_encoder.encode(None)? {
            vvenc::EncoderOutput::None => {}
            vvenc::EncoderOutput::Data(data, done) => {
                println!("got output frame {}", output_frame_num);
                assert_eq!(output.write(data.payload)?, data.payload.len());
                output_frame_num += 1;
                encoding_done = done;
            }
        }
    }

    Ok(())
}
