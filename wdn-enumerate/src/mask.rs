use image::{Rgba, RgbaImage};

pub const TILE_WIDTH: u32 = 200;
pub const TILE_HEIGHT: u32 = 400;

const MASK_COLOR: Rgba<u8> = Rgba([0x33, 0x3c, 0x43, 0xff]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    /// The top sprite has no masked pixels at all.
    Empty,
    /// No pixels in either sprite overlap or touch.
    Disjoint,
    /// Some or all pixels in the top sprite are present in the base sprite, or the two sprites touch.
    Overlap,
}

/// Per-tile masks of a row-major spritesheet, keeping only pixels matching `MASK_COLOR`.
pub struct SpriteMask {
    source: RgbaImage,
    columns: u32,
    tiles: Vec<Box<[bool]>>,
}

impl SpriteMask {
    /// Loads `source`, masks it down to `MASK_COLOR` pixels, writes the result to `masked_out`,
    /// and splits it into per-tile boolean grids for classification.
    pub fn load(source: &str, masked_out: &str) -> SpriteMask {
        let image = image::open(source)
            .unwrap_or_else(|err| panic!("failed to open {source}: {err}"))
            .to_rgba8();

        let columns = image.width() / TILE_WIDTH;
        let rows = image.height() / TILE_HEIGHT;

        let mut masked = RgbaImage::new(image.width(), image.height());
        let mut tiles = vec![
            vec![false; (TILE_WIDTH * TILE_HEIGHT) as usize].into_boxed_slice();
            (columns * rows) as usize
        ];

        for (x, y, pixel) in image.enumerate_pixels() {
            if *pixel != MASK_COLOR {
                continue;
            }
            masked.put_pixel(x, y, *pixel);

            let column = x / TILE_WIDTH;
            let row = y / TILE_HEIGHT;
            let local_x = x % TILE_WIDTH;
            let local_y = y % TILE_HEIGHT;
            tiles[(row * columns + column) as usize][(local_y * TILE_WIDTH + local_x) as usize] =
                true;
        }

        masked
            .save(masked_out)
            .unwrap_or_else(|err| panic!("failed to write {masked_out}: {err}"));

        SpriteMask {
            source: image,
            columns,
            tiles,
        }
    }

    fn pixel(&self, id: usize, x: u32, y: u32) -> bool {
        self.tiles[id][(y * TILE_WIDTH + x) as usize]
    }

    fn touches_base(&self, base_id: usize, x: u32, y: u32) -> bool {
        (x > 0 && self.pixel(base_id, x - 1, y))
            || (y > 0 && self.pixel(base_id, x, y - 1))
            || (x + 1 < TILE_WIDTH && self.pixel(base_id, x + 1, y))
            || (y + 1 < TILE_HEIGHT && self.pixel(base_id, x, y + 1))
    }

    pub fn classify(&self, base_id: usize, top_id: usize) -> Classification {
        let mut top_count = 0usize;
        let mut overlap_count = 0usize;
        let mut touching = false;

        for y in 0..TILE_HEIGHT {
            for x in 0..TILE_WIDTH {
                if !self.pixel(top_id, x, y) {
                    continue;
                }
                top_count += 1;

                if self.pixel(base_id, x, y) {
                    overlap_count += 1;
                } else if !touching && self.touches_base(base_id, x, y) {
                    touching = true;
                }
            }
        }

        if top_count == 0 {
            Classification::Empty
        } else if overlap_count > 0 || touching {
            Classification::Overlap
        } else {
            Classification::Disjoint
        }
    }

    fn tile_origin(&self, id: usize) -> (u32, u32) {
        let id = id as u32;
        ((id % self.columns) * TILE_WIDTH, (id / self.columns) * TILE_HEIGHT)
    }

    /// Composites the original (unmasked) base tile with the top tile drawn over it.
    pub fn composite(&self, base_id: usize, top_id: usize) -> RgbaImage {
        let (base_x, base_y) = self.tile_origin(base_id);
        let (top_x, top_y) = self.tile_origin(top_id);

        let mut tile = RgbaImage::new(TILE_WIDTH, TILE_HEIGHT);
        for y in 0..TILE_HEIGHT {
            for x in 0..TILE_WIDTH {
                let base = *self.source.get_pixel(base_x + x, base_y + y);
                let top = *self.source.get_pixel(top_x + x, top_y + y);
                tile.put_pixel(x, y, over(top, base));
            }
        }
        tile
    }
}

/// Standard "source over destination" alpha compositing.
fn over(src: Rgba<u8>, dst: Rgba<u8>) -> Rgba<u8> {
    let src_a = src.0[3] as f32 / 255.0;
    if src_a <= 0.0 {
        return dst;
    }
    if src_a >= 1.0 {
        return src;
    }

    let dst_a = dst.0[3] as f32 / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);
    let mut out = [0u8; 4];
    for channel in 0..3 {
        let src_c = src.0[channel] as f32 / 255.0;
        let dst_c = dst.0[channel] as f32 / 255.0;
        let out_c = if out_a > 0.0 {
            (src_c * src_a + dst_c * dst_a * (1.0 - src_a)) / out_a
        } else {
            0.0
        };
        out[channel] = (out_c * 255.0).round() as u8;
    }
    out[3] = (out_a * 255.0).round() as u8;
    Rgba(out)
}
