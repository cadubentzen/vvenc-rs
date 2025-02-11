use vvenc::*;

#[test]
fn basic() {
    let config = Config::new(160, 120, 24, 0, 32, Preset::Faster);
    let encoder = Encoder::new(config).unwrap();
}
