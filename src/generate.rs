use crate::analyze::{Extents, Nearby, PixelKind};
use crate::sizes::{KicadDim, KicadPos, PixelDim, PixelPos};
use image::{DynamicImage, GenericImageView};
use rand::Rng;
use std::io;
use std::io::Write;
use std::ops::Neg;

pub struct Config {
    pub pixel_pitch: KicadDim,
    pub clearance: KicadDim,
}

pub fn output_file(
    name: &str,
    image: DynamicImage,
    config: Config,
    mut r: impl Rng,
    mut w: impl Write,
) -> Result<(), io::Error> {
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
        sexpr(w, "generator", |w| w.write_all(b"silkgen"))?;
        sexpr(w, "layer", |w| w.write_all(b"F.Silkscreen"))?;
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
            let layers: &[_] = match PixelKind::from_pixel(pixel) {
                PixelKind::Transparent => continue,
                PixelKind::Light => &["F.SilkS"],
                PixelKind::Dark => &["F.Cu", "F.Mask"],
            };
            let nearby = Nearby::from_index(&image, x, y);
            for layer in layers {
                draw_pixel(
                    w,
                    &mut r,
                    PixelPos {
                        x: PixelDim(x),
                        y: PixelDim(y),
                    },
                    &extents,
                    &nearby,
                    &config,
                    layer,
                )?;
            }
        }

        Ok(())
    })
}

fn draw_pixel(
    w: &mut impl Write,
    r: &mut impl Rng,
    top_left: PixelPos,
    extents: &Extents,
    nearby: &Nearby,
    config: &Config,
    layer: &str,
) -> Result<(), io::Error> {
    sexpr(w, "fp_poly", |w| {
        sexpr(w, "pts", |w| {
            // shift by 1 since we base positions on the top left of the pixel
            let center = extents.center() + PixelPos::X1 + PixelPos::Y1;
            // find pixel coords of edges
            let top = top_left.y;
            let bot = top + PixelDim(1);
            let left = top_left.x;
            let right = left + PixelDim(1);
            // find kicad coords of edges
            let kicad_pos = |pos: PixelDim, center_pos: PixelDim| {
                neg_if(
                    pos < center_pos,
                    config.pixel_pitch * pos.abs_diff(center_pos),
                )
            };
            let top = kicad_pos(top, center.y);
            let bot = kicad_pos(bot, center.y);
            let left = kicad_pos(left, center.x);
            let right = kicad_pos(right, center.x);
            // adjust to apply clearance to nearby edges
            let apply_clearance = |pos, adjacent_kind, dir_is_positive| {
                if (nearby.this, adjacent_kind) == (PixelKind::Light, PixelKind::Dark) {
                    // only light-> dark transitions need to offset the (light) silkscreen
                    if dir_is_positive {
                        pos - config.clearance
                    } else {
                        pos + config.clearance
                    }
                } else {
                    pos
                }
            };
            let top = apply_clearance(top, nearby.top, false);
            let bot = apply_clearance(bot, nearby.bot, true);
            let left = apply_clearance(left, nearby.left, false);
            let right = apply_clearance(right, nearby.right, true);
            // place corners of the polygon
            xy(w, KicadPos { x: left, y: top })?;
            xy(w, KicadPos { x: right, y: top })?;
            xy(w, KicadPos { x: right, y: bot })?;
            xy(w, KicadPos { x: left, y: bot })?;
            Ok(())
        })?;
        sexpr(w, "layer", |w| w.write_all(layer.as_bytes()))?;
        sexpr(w, "width", |w| w.write_all(b"0"))?;
        sexpr(w, "fill", |w| w.write_all(b"solid"))?;
        tstamp(w, r)
    })
}

fn xy(w: &mut impl Write, pos: KicadPos) -> Result<(), io::Error> {
    sexpr(w, "xy", |w| write!(w, "{} {}", pos.x, pos.y))
}

fn tstamp(w: &mut impl Write, r: &mut impl Rng) -> Result<(), io::Error> {
    sexpr(w, "tstamp", |w| {
        let uuid = uuid::Builder::from_random_bytes(r.gen()).into_uuid();
        w.write_all(uuid.to_string().as_bytes())
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

fn neg_if<T: Neg<Output = T>>(cond: bool, x: T) -> T {
    if cond {
        -x
    } else {
        x
    }
}
