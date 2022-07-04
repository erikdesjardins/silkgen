use crate::opt::Config;
use crate::sizes::{KicadPos, PixelDim, PixelPos};
use image::{GenericImageView, LumaA, Pixel};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PixelKind {
    Light,
    Dark,
}

impl PixelKind {
    pub fn from_pixel(pixel: impl Pixel<Subpixel = u8>) -> Option<Self> {
        let LumaA([luma, alpha]) = pixel.to_luma_alpha();

        match () {
            _ if alpha < u8::MAX / 2 => None,
            _ if luma > u8::MAX / 2 => Some(Self::Light),
            _ => Some(Self::Dark),
        }
    }
}

#[derive(Debug)]
pub struct Extents {
    pub min: PixelPos,
    pub max: PixelPos,
}

impl Extents {
    pub fn from_image(image: &impl GenericImageView<Pixel = impl Pixel<Subpixel = u8>>) -> Self {
        let mut min_x = u32::MAX;
        let mut max_x = 0;
        let mut min_y = u32::MAX;
        let mut max_y = 0;

        for (x, y, pixel) in image.pixels() {
            if PixelKind::from_pixel(pixel).is_some() {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }

        Self {
            min: PixelPos {
                x: PixelDim(min_x),
                y: PixelDim(min_y),
            },
            max: PixelPos {
                x: PixelDim(max_x),
                y: PixelDim(max_y),
            },
        }
    }

    pub fn center(&self) -> PixelPos {
        let x = self.min.x.0 + (self.max.x.0 - self.min.x.0) / 2;
        let y = self.min.y.0 + (self.max.y.0 - self.min.y.0) / 2;
        PixelPos {
            x: PixelDim(x),
            y: PixelDim(y),
        }
    }
}

#[derive(Debug)]
pub struct Nearby {
    pub top: Option<PixelKind>,
    pub bot: Option<PixelKind>,
    pub left: Option<PixelKind>,
    pub right: Option<PixelKind>,
    pub top_left: Option<PixelKind>,
    pub top_right: Option<PixelKind>,
    pub bot_left: Option<PixelKind>,
    pub bot_right: Option<PixelKind>,
}

impl Nearby {
    pub fn from_pos(
        image: &impl GenericImageView<Pixel = impl Pixel<Subpixel = u8>>,
        PixelPos {
            x: PixelDim(x),
            y: PixelDim(y),
        }: PixelPos,
    ) -> Self {
        let try_get = |x, y| {
            let x = x?;
            let y = y?;
            if image.in_bounds(x, y) {
                PixelKind::from_pixel(image.get_pixel(x, y))
            } else {
                None
            }
        };
        Self {
            top: try_get(Some(x), y.checked_sub(1)),
            bot: try_get(Some(x), y.checked_add(1)),
            left: try_get(x.checked_sub(1), Some(y)),
            right: try_get(x.checked_add(1), Some(y)),
            top_left: try_get(x.checked_sub(1), y.checked_sub(1)),
            top_right: try_get(x.checked_add(1), y.checked_sub(1)),
            bot_left: try_get(x.checked_sub(1), y.checked_add(1)),
            bot_right: try_get(x.checked_add(1), y.checked_add(1)),
        }
    }
}

pub fn for_each_point_in_pixel<E>(
    top_left: PixelPos,
    kind: PixelKind,
    nearby: &Nearby,
    extents: &Extents,
    config: &Config,
    mut f: impl FnMut(KicadPos) -> Result<(), E>,
) -> Result<(), E> {
    // shift by 1 since we base positions on the top left of the pixel
    let center = extents.center() + PixelPos::X1 + PixelPos::Y1;
    // find pixel coords of edges
    let top = top_left.y;
    let bot = top + PixelDim(1);
    let left = top_left.x;
    let right = left + PixelDim(1);
    // find kicad coords of edges
    let kicad_pos = |pos: PixelDim, center_pos: PixelDim| {
        let relative_pos = config.pixel_pitch * pos.abs_diff(center_pos);
        if pos < center_pos {
            -relative_pos
        } else {
            relative_pos
        }
    };
    let top = kicad_pos(top, center.y);
    let bot = kicad_pos(bot, center.y);
    let left = kicad_pos(left, center.x);
    let right = kicad_pos(right, center.x);
    // place points
    let mut add_points_from =
        |mut x, mut y, horiz, vert, diag, horiz_is_positive, vert_is_positive| {
            let sub_or_add = |lhs, should_sub, rhs| {
                if should_sub {
                    lhs - rhs
                } else {
                    lhs + rhs
                }
            };

            match kind {
                PixelKind::Dark => {
                    // dark pixels always fill the entire pixel, and don't need special processing
                    f(KicadPos { x, y })
                }
                PixelKind::Light => {
                    // prepare inset dimensions
                    let x_inset = sub_or_add(x, horiz_is_positive, config.clearance);
                    let y_inset = sub_or_add(y, vert_is_positive, config.clearance);

                    // add clearance for directly adjacent dark->light pixel transitions
                    if horiz == Some(PixelKind::Dark) {
                        x = x_inset;
                    }
                    if vert == Some(PixelKind::Dark) {
                        y = y_inset;
                    }

                    if x == x_inset || y == y_inset || diag != Some(PixelKind::Dark) {
                        // normal case: no diagonal inclusion, or already inset on one side or the other
                        f(KicadPos { x, y })
                    } else {
                        // special case: handle diagonal inclusion, splitting the corner into three points
                        //         x  x_inset
                        //
                        // y       +  +---- ...
                        //            |
                        // y_inset +--+
                        //         |
                        //         |
                        //        ...
                        let mut new_points = [
                            KicadPos { x, y: y_inset },
                            KicadPos {
                                x: x_inset,
                                y: y_inset,
                            },
                            KicadPos { x: x_inset, y },
                        ];
                        // handle different coordinate ordering for top right and bottom left
                        if horiz_is_positive != vert_is_positive {
                            new_points.reverse();
                        }
                        for point in new_points {
                            f(point)?;
                        }
                        Ok(())
                    }
                }
            }
        };
    add_points_from(
        left,
        top,
        nearby.left,
        nearby.top,
        nearby.top_left,
        false,
        false,
    )?;
    add_points_from(
        right,
        top,
        nearby.right,
        nearby.top,
        nearby.top_right,
        true,
        false,
    )?;
    add_points_from(
        right,
        bot,
        nearby.right,
        nearby.bot,
        nearby.bot_right,
        true,
        true,
    )?;
    add_points_from(
        left,
        bot,
        nearby.left,
        nearby.bot,
        nearby.bot_left,
        false,
        true,
    )?;
    Ok(())
}
