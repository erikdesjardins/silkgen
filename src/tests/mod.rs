use crate::generate;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[test]
fn basic() {
    insta::assert_snapshot!(run_against(include_bytes!("basic.png")));
}

fn run_against(file: &[u8]) -> String {
    let image = image::load_from_memory(file).unwrap();

    let pixel_pitch = "1mm".parse().unwrap();

    let rng = StdRng::from_seed(*b"1234567890abcdefghijklmnopqrstuv");

    let mut out = Vec::new();

    generate::output_file("testname", image, pixel_pitch, rng, &mut out).unwrap();

    String::from_utf8(out).unwrap()
}
