use crate::generate;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn run_against(file: &[u8]) -> String {
    let image = image::load_from_memory(file).unwrap();

    let config = generate::Config {
        pixel_pitch: "1mm".parse().unwrap(),
        clearance: "0.1mm".parse().unwrap(),
    };

    let rng = StdRng::from_seed(*b"1234567890abcdefghijklmnopqrstuv");

    let mut out = Vec::new();

    generate::output_file("testname", image, config, rng, &mut out).unwrap();

    String::from_utf8(out).unwrap()
}

#[test]
fn basic() {
    insta::assert_snapshot!(run_against(include_bytes!("basic.png")));
}

#[test]
fn clearance() {
    insta::assert_snapshot!(run_against(include_bytes!("clearance.png")));
}

#[test]
fn annoying_dog() {
    insta::assert_snapshot!(run_against(include_bytes!("annoying_dog.png")));
}
