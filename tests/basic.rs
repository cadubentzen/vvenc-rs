use vvenc::*;

#[test]
fn basic() {
    const WIDTH: i32 = 160;
    const HEIGHT: i32 = 120;
    let config = Config::new(WIDTH, HEIGHT, 24, 0, 32, Preset::Faster);
    let mut encoder = Encoder::new(config).unwrap();

    let y_size = (WIDTH * HEIGHT) as usize;
    let uv_size = (WIDTH * HEIGHT / 4) as usize;
    let y = vec![0i16; y_size];
    let u = vec![0i16; uv_size];
    let v = vec![0i16; uv_size];

    let frame = Frame {
        planes: [
            Plane {
                data: y.as_slice(),
                width: WIDTH,
                height: HEIGHT,
                stride: WIDTH,
            },
            Plane {
                data: u.as_slice(),
                width: WIDTH / 2,
                height: HEIGHT / 2,
                stride: WIDTH / 2,
            },
            Plane {
                data: v.as_slice(),
                width: WIDTH / 2,
                height: HEIGHT / 2,
                stride: WIDTH / 2,
            },
        ],
        sequence_number: 0,
        cts: None,
    };

    assert!(encoder.encode(frame).unwrap().is_none());
    let au = encoder.flush().unwrap().unwrap();
    assert!(au.inner.payloadUsedSize > 0);
    assert!(encoder.flush().unwrap_err() == Error::RestartRequired);
}
