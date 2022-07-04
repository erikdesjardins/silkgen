use crate::generate;

fn run_against(name: &str, file: &[u8]) -> String {
    let image = image::load_from_memory(file).unwrap();

    let config = generate::Config {
        pixel_pitch: "1mm".parse().unwrap(),
        clearance: "0.1mm".parse().unwrap(),
    };

    let mut out = Vec::new();

    generate::output_file(name, image, config, &mut out).unwrap();

    String::from_utf8(out).unwrap()
}

macro_rules! test {
    ($name:ident, $file:literal) => {
        #[test]
        fn $name() {
            insta::assert_snapshot!(run_against(stringify!($name), include_bytes!($file)));
        }
    };
}

test!(basic, "basic.png");
test!(clearance, "clearance.png");
test!(annoying_dog, "annoying_dog.png");
