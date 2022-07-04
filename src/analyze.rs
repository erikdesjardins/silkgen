use crate::sizes::{PixelDim, PixelPos};
use image::{GenericImageView, GrayAlphaImage, LumaA};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PixelKind {
    Transparent,
    Light,
    Dark,
}

impl PixelKind {
    pub fn from_pixel(pixel: LumaA<u8>) -> Self {
        let LumaA([luma, alpha]) = pixel;

        match () {
            _ if alpha < u8::MAX / 2 => Self::Transparent,
            _ if luma > u8::MAX / 2 => Self::Light,
            _ => Self::Dark,
        }
    }

    pub fn is_significant(&self) -> bool {
        match self {
            Self::Transparent => false,
            Self::Light | Self::Dark => true,
        }
    }
}

#[derive(Debug)]
pub struct Extents {
    pub min: PixelPos,
    pub max: PixelPos,
}

impl Extents {
    pub fn from_image(image: &GrayAlphaImage) -> Self {
        let mut min_x = u32::MAX;
        let mut max_x = 0;
        let mut min_y = u32::MAX;
        let mut max_y = 0;

        for (x, y, pixel) in GenericImageView::pixels(image) {
            if PixelKind::from_pixel(pixel).is_significant() {
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
    pub this: PixelKind,
    pub top: PixelKind,
    pub bot: PixelKind,
    pub left: PixelKind,
    pub right: PixelKind,
    pub top_left: PixelKind,
    pub top_right: PixelKind,
    pub bot_left: PixelKind,
    pub bot_right: PixelKind,
}

impl Nearby {
    pub fn from_index(image: &GrayAlphaImage, x: u32, y: u32) -> Self {
        let try_get = |x, y| {
            let go = || image.get_pixel_checked(x?, y?);
            match go() {
                Some(pixel) => PixelKind::from_pixel(*pixel),
                None => PixelKind::Transparent,
            }
        };
        Self {
            this: try_get(Some(x), Some(y)),
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
