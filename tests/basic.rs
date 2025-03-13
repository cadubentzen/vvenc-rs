use vvenc::*;

struct BasicLogger;

impl Logger for BasicLogger {
    fn log(&self, level: LogLevel, message: &str) {
        println!("{:?}: {}", level, message);
    }
}

#[test]
fn basic() {
    const WIDTH: i32 = 160;
    const HEIGHT: i32 = 120;
    const CHROMA_FORMAT: ChromaFormat = ChromaFormat::Chroma420;

    let mut config = Config::default();
    config
        .set_width(WIDTH)
        .set_height(HEIGHT)
        .set_framerate(Rational { num: 30, den: 1 })
        .set_qp(Qp::new(32).unwrap())
        .set_internal_chroma_format(CHROMA_FORMAT)
        .set_log_level(LogLevel::Details)
        .set_logger(Box::new(BasicLogger))
        .set_preset(Preset::Faster)
        .unwrap();

    let mut encoder = Encoder::with_config(config).unwrap();
    let mut data = vec![0u8; (2 * WIDTH * HEIGHT + 1024) as usize];

    let y_size = (WIDTH * HEIGHT) as usize;
    let uv_size = (WIDTH * HEIGHT / 4) as usize;
    let y = vec![0i16; y_size];
    let u = vec![0i16; uv_size];
    let v = vec![0i16; uv_size];

    let mut buffer = YUVBuffer::new(WIDTH, HEIGHT, CHROMA_FORMAT);
    buffer
        .plane_mut(YUVComponent::Y)
        .data_mut()
        .copy_from_slice(&y);
    buffer
        .plane_mut(YUVComponent::U)
        .data_mut()
        .copy_from_slice(&u);
    buffer
        .plane_mut(YUVComponent::V)
        .data_mut()
        .copy_from_slice(&v);

    buffer.set_cts(0);
    buffer.set_opaque(1234u64);
    assert!(encoder.encode(&mut buffer, &mut data).unwrap().is_none());

    buffer.set_cts(1);
    buffer.set_opaque(5678u64);
    assert!(encoder.encode(&mut buffer, &mut data).unwrap().is_none());
    let (mut au, encode_done) = encoder.flush(&mut data).unwrap().unwrap();
    // println!("AU: {:?}", au);
    assert!(au.payload().len() > 0);
    assert!(!encode_done);
    assert_eq!(*au.take_opaque(), 5678u64);
    assert_eq!(au.cts().unwrap(), 1);
    // assert_eq!(au.cts().unwrap(), 0);
    let (mut au, encode_done) = encoder.flush(&mut data).unwrap().unwrap();
    // println!("AU: {:?}", au);
    assert!(au.payload().len() > 0);
    assert!(encode_done);
    assert_eq!(*au.take_opaque(), 1234u64);
    assert_eq!(au.cts().unwrap(), 0);
    assert!(encoder.flush(&mut data).unwrap_err() == Error::RestartRequired);
}
