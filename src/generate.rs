use crate::analyze::{for_each_point_in_pixel, Extents, Nearby, PixelKind};
use crate::opt::Config;
use crate::sizes::{PixelDim, PixelPos};
use image::{DynamicImage, GenericImageView};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use sha2::{Digest, Sha256};
use std::io;
use std::io::Write;

pub fn output_file(
    name: &str,
    image: DynamicImage,
    config: &Config,
    mut w: impl Write,
) -> Result<(), io::Error> {
    let mut r = {
        // create deterministic hasher based on file name
        let mut hasher = Sha256::new();
        hasher.update(name);
        ChaCha12Rng::from_seed(hasher.finalize().try_into().unwrap())
    };

    let image = image.into_luma_alpha8();

    log::info!(
        "Total image dimensions: {} x {}",
        image.width(),
        image.height()
    );

    let extents = Extents::from_image(&image);

    log::info!("Extent of significant pixels: {:?}", extents);
    log::info!("Center of significant pixels: {:?}", extents.center());

    sexpr(&mut w, "footprint", |w| {
        w.write_all(b"\"")?;
        w.write_all(name.replace('\"', "").as_bytes())?;
        w.write_all(b"\"\n")?;

        // prelude
        sexpr(w, "version", |w| w.write_all(b"20220630"))?;
        sexpr(w, "generator", |w| {
            w.write_all(b"\"")?;
            w.write_all(
                concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION")).as_bytes(),
            )?;
            w.write_all(b"\"")
        })?;
        sexpr(w, "layer", |w| w.write_all(b"F.SilkS"))?;
        sexpr(w, "tedit", |w| w.write_all(b"0"))?;
        sexpr(w, "attr", |w| {
            w.write_all(b"board_only exclude_from_pos_files exclude_from_bom")
        })?;

        // text references
        sexpr(w, "fp_text", |w| {
            w.write_all(b"reference \"G***\" (at 0 0) (layer F.Fab)\n")?;
            w.write_all(b"(effects (font (size 1.524 1.524) (thickness 0.3)))\n")?;
            tstamp(w, &mut r)
        })?;
        sexpr(w, "fp_text", |w| {
            w.write_all(b"value \"LOGO\" (at 0.75 0) (layer F.Fab) hide\n")?;
            w.write_all(b"(effects (font (size 1.524 1.524) (thickness 0.3)))\n")?;
            tstamp(w, &mut r)
        })?;

        // pixels
        for (x, y, pixel) in GenericImageView::pixels(&image) {
            let kind = match PixelKind::from_pixel(pixel) {
                Some(k) => k,
                None => continue,
            };
            let layers: &[_] = match kind {
                PixelKind::Light => &["F.SilkS"],
                PixelKind::Dark => &["F.Cu", "F.Mask"],
            };
            let nearby = Nearby::from_index(&image, x, y);
            for layer in layers {
                sexpr(w, "fp_poly", |w| {
                    sexpr(w, "pts", |w| {
                        for_each_point_in_pixel(
                            PixelPos {
                                x: PixelDim(x),
                                y: PixelDim(y),
                            },
                            kind,
                            &nearby,
                            &extents,
                            &config,
                            |point| sexpr(w, "xy", |w| write!(w, "{} {}", point.x, point.y)),
                        )
                    })?;
                    sexpr(w, "layer", |w| w.write_all(layer.as_bytes()))?;
                    sexpr(w, "width", |w| w.write_all(b"0"))?;
                    sexpr(w, "fill", |w| w.write_all(b"solid"))?;
                    tstamp(w, &mut r)
                })?;
            }
        }

        Ok(())
    })
}

fn tstamp(w: &mut impl Write, r: &mut impl Rng) -> Result<(), io::Error> {
    sexpr(w, "tstamp", |w| {
        let uuid = uuid::Builder::from_random_bytes(r.gen()).into_uuid();
        write!(w, "{}", uuid)
    })
}

fn sexpr<W: Write, R>(
    w: &mut W,
    name: &str,
    f: impl FnOnce(&mut W) -> Result<R, io::Error>,
) -> Result<R, io::Error> {
    w.write_all(b"(")?;
    w.write_all(name.as_bytes())?;
    w.write_all(b" ")?;
    let r = f(w)?;
    w.write_all(b")\n")?;
    Ok(r)
}
