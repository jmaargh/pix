// raster.rs    Raster images.
//
// Copyright (c) 2017-2020  Douglas P Lau
// Copyright (c) 2019-2020  Jeron Aldaron Lau
//
use crate::chan::{Ch16, Ch8};
use crate::el::Pixel;
use crate::ops::PorterDuff;
use std::convert::TryFrom;
use std::slice::{from_raw_parts_mut, ChunksExact, ChunksExactMut};

/// Image arranged as a rectangular array of pixels.
///
/// A `Raster` can be constructed using one of the *with_* methods:
/// * [with_clear](#method.with_clear)
/// * [with_color](#method.with_color)
/// * [with_raster](#method.with_raster)
/// * [with_pixels](#method.with_pixels)
/// * [with_u8_buffer](#method.with_u8_buffer)
/// * [with_u16_buffer](#method.with_u16_buffer)
///
/// ### Create a clear `Raster`
/// ```
/// # use pix::*;
/// let r = Raster::<SRgb8>::with_clear(100, 100);
/// ```
///
/// ### Create a `Raster` with a solid color rectangle
/// ```
/// # use pix::*;
/// # use pix::ops::Source;
/// let mut r = Raster::<SRgb8>::with_clear(10, 10);
/// let clr = SRgb8::new(0xFF, 0xFF, 0x00);
/// r.composite_color((2, 4, 3, 3), clr, Source);
/// ```
pub struct Raster<P: Pixel> {
    width: i32,
    height: i32,
    pixels: Box<[P]>,
}

/// `Iterator` of *rows* in a [raster], as slices of [pixel]s.
///
/// This struct is created by the [rows] method of [Raster].
///
/// [pixel]: el/trait.Pixel.html
/// [raster]: struct.Raster.html
/// [rows]: struct.Raster.html#method.rows
pub struct Rows<'a, P: Pixel> {
    chunks: ChunksExact<'a, P>,
}

/// `Iterator` of *rows* in a [raster], as mutable slices of [pixel]s.
///
/// This struct is created by the [rows_mut] method of [Raster].
///
/// [pixel]: el/trait.Pixel.html
/// [raster]: struct.Raster.html
/// [rows_mut]: struct.Raster.html#method.rows_mut
pub struct RowsMut<'a, P: Pixel> {
    chunks: ChunksExactMut<'a, P>,
}

/// Location / dimensions of pixels relative to a [Raster](struct.Raster.html).
///
/// ### Create directly
/// ```
/// use pix::Region;
///
/// let r0 = Region::new(80, 20, 120, 280);
/// let r1 = r0.intersection((50, 40, 360, 240));
/// ```
/// ### Create from Raster
/// ```
/// use pix::{Raster, SRgb8};
///
/// let r = Raster::<SRgb8>::with_clear(100, 100);
/// let reg = r.region(); // (0, 0, 100, 100)
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl<P: Pixel> Into<Box<[P]>> for Raster<P> {
    /// Get internal pixel data as boxed slice.
    fn into(self) -> Box<[P]> {
        self.pixels
    }
}

impl<P: Pixel> Into<Vec<P>> for Raster<P> {
    /// Get internal pixel data as `Vec` of pixels.
    fn into(self) -> Vec<P> {
        self.pixels.into()
    }
}

impl<P: Pixel> Raster<P> {
    /// Construct a `Raster` with all pixels set to the default value.
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is greater than `std::i32::MAX`.
    ///
    /// ## Examples
    /// ```
    /// # use pix::*;
    /// let r1 = Raster::<SGray8>::with_clear(20, 20);
    /// let r2 = Raster::<Mask8>::with_clear(64, 64);
    /// let r3 = Raster::<SRgb16>::with_clear(10, 10);
    /// let r4 = Raster::<SGraya32>::with_clear(100, 250);
    /// ```
    pub fn with_clear(width: u32, height: u32) -> Self {
        Self::with_color(width, height, P::default())
    }

    /// Construct a `Raster` with all pixels set to one color.
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is greater than `std::i32::MAX`.
    ///
    /// ## Example
    /// ```
    /// use pix::{Raster, SRgb8};
    ///
    /// let clr = SRgb8::new(0x40, 0xAA, 0xBB);
    /// let r = Raster::<SRgb8>::with_color(15, 15, clr);
    /// ```
    pub fn with_color(width: u32, height: u32, clr: P) -> Self {
        let width = i32::try_from(width).unwrap();
        let height = i32::try_from(height).unwrap();
        let len = (width * height) as usize;
        let pixels = vec![clr; len].into_boxed_slice();
        Raster {
            width,
            height,
            pixels,
        }
    }

    /// Construct a `Raster` with another `Raster`.
    ///
    /// The pixel format can be converted using this method.
    ///
    /// * `S` `Pixel` format of source `Raster`.
    ///
    /// ### Convert from SRgb8 to Rgba16
    /// ```
    /// use pix::{Raster, Rgba16, SRgb8};
    ///
    /// let mut r0 = Raster::<SRgb8>::with_clear(50, 50);
    /// // load pixels into raster
    /// let r1 = Raster::<Rgba16>::with_raster(&r0);
    /// ```
    pub fn with_raster<S>(src: &Raster<S>) -> Self
    where
        S: Pixel,
        P::Chan: From<S::Chan>,
    {
        let mut r = Raster::with_clear(src.width(), src.height());
        let srows = src.rows();
        let drows = r.rows_mut();
        for (drow, srow) in drows.zip(srows) {
            for (d, s) in drow.iter_mut().zip(srow) {
                *d = s.convert();
            }
        }
        r
    }

    /// Construct a `Raster` with owned pixel data.  You can get ownership of
    /// the pixel data back from the `Raster` as either a `Vec<P>` or a
    /// `Box<[P]>` by calling `into()`.
    ///
    /// * `B` Owned pixed type (`Vec` or boxed slice).
    /// * `width` Width of `Raster`.
    /// * `height` Height of `Raster`.
    /// * `pixels` Pixel data.
    ///
    /// # Panics
    ///
    /// * If `width` or `height` is greater than `std::i32::MAX`
    /// * If `pixels` length is not equal to `width` * `height`
    ///
    /// ## Example
    /// ```
    /// # use pix::*;
    /// # use pix::ops::Source;
    /// let p = vec![Rgb8::new(255, 0, 255); 16];     // vec of magenta pix
    /// let mut r = Raster::with_pixels(4, 4, p);     // convert to raster
    /// let clr = Rgb8::new(0x00, 0xFF, 0x00);        // green
    /// r.composite_color((2, 0, 1, 3), clr, Source); // make stripe
    /// let p2 = Into::<Vec<Rgb8>>::into(r);          // back to vec
    /// ```
    pub fn with_pixels<B>(width: u32, height: u32, pixels: B) -> Self
    where
        B: Into<Box<[P]>>,
    {
        let width = i32::try_from(width).unwrap();
        let height = i32::try_from(height).unwrap();
        let len = (width * height) as usize;
        let pixels = pixels.into();
        assert_eq!(len, pixels.len());
        Raster {
            width,
            height,
            pixels,
        }
    }

    /// Construct a `Raster` from a `u8` buffer.
    ///
    /// * `B` Owned pixed type (`Vec` or boxed slice).
    /// * `width` Width of `Raster`.
    /// * `height` Height of `Raster`.
    /// * `buffer` Buffer of pixel data.
    ///
    /// # Panics
    ///
    /// * If `width` or `height` is greater than `std::i32::MAX`
    /// * If `buffer` length is not equal to `width` * `height` *
    ///   `std::mem::size_of::<P>()`
    pub fn with_u8_buffer<B>(width: u32, height: u32, buffer: B) -> Self
    where
        B: Into<Box<[u8]>>,
        P: Pixel<Chan = Ch8>,
    {
        let width = i32::try_from(width).unwrap();
        let height = i32::try_from(height).unwrap();
        let len = (width * height) as usize;
        assert!(len > 0);
        let buffer: Box<[u8]> = buffer.into();
        let capacity = buffer.len();
        assert_eq!(
            len * std::mem::size_of::<P>(),
            capacity * std::mem::size_of::<u8>()
        );
        let slice = Box::<[u8]>::into_raw(buffer);
        let pixels: Box<[P]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut P;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Raster {
            width,
            height,
            pixels,
        }
    }

    /// Construct a `Raster` from a `u16` buffer.
    ///
    /// * `B` Owned pixed type (`Vec` or boxed slice).
    /// * `width` Width of `Raster`.
    /// * `height` Height of `Raster`.
    /// * `buffer` Buffer of pixel data (in native-endian byte order).
    ///
    /// # Panics
    ///
    /// * If `width` or `height` is greater than `std::i32::MAX`
    /// * If `buffer` length is not equal to `width` * `height` *
    ///   `std::mem::size_of::<P>()`
    pub fn with_u16_buffer<B>(width: u32, height: u32, buffer: B) -> Self
    where
        B: Into<Box<[u16]>>,
        P: Pixel<Chan = Ch16>,
    {
        let width = i32::try_from(width).unwrap();
        let height = i32::try_from(height).unwrap();
        let len = (width * height) as usize;
        assert!(len > 0);
        let buffer: Box<[u16]> = buffer.into();
        let capacity = buffer.len();
        assert_eq!(
            len * std::mem::size_of::<P>(),
            capacity * std::mem::size_of::<u16>()
        );
        let slice = Box::<[u16]>::into_raw(buffer);
        let pixels: Box<[P]> = unsafe {
            let ptr = (*slice).as_mut_ptr() as *mut P;
            Box::from_raw(from_raw_parts_mut(ptr, len))
        };
        Raster {
            width,
            height,
            pixels,
        }
    }

    /// Get width of `Raster`.
    pub fn width(&self) -> u32 {
        self.width as u32
    }

    /// Get height of `Raster`.
    pub fn height(&self) -> u32 {
        self.height as u32
    }

    /// Clear all pixels to default value.
    pub fn clear(&mut self) {
        for p in self.pixels.iter_mut() {
            *p = P::default();
        }
    }

    /// Get one pixel.
    pub fn pixel(&self, x: i32, y: i32) -> P {
        debug_assert!(x >= 0 && x < self.width);
        debug_assert!(y >= 0 && y < self.height);
        let i = (self.width * y + x) as usize;
        self.pixels[i]
    }

    /// Get a mutable pixel.
    pub fn pixel_mut(&mut self, x: i32, y: i32) -> &mut P {
        debug_assert!(x >= 0 && x < self.width);
        debug_assert!(y >= 0 && y < self.height);
        let i = (self.width * y + x) as usize;
        &mut self.pixels[i]
    }

    /// Get a slice of all pixels.
    pub fn pixels(&self) -> &[P] {
        &self.pixels
    }

    /// Get a mutable slice of all pixels.
    pub fn pixels_mut(&mut self) -> &mut [P] {
        &mut self.pixels
    }

    /// Get an `Iterator` of rows within a `Raster`.
    pub fn rows(&self) -> Rows<P> {
        Rows::new(self)
    }

    /// Get an `Iterator` of mutable rows within a `Raster`.
    pub fn rows_mut(&mut self) -> RowsMut<P> {
        RowsMut::new(self)
    }

    /// Get `Region` of entire `Raster`.
    pub fn region(&self) -> Region {
        Region::new(0, 0, self.width(), self.height())
    }

    /// Get intersection with a `Region`
    pub fn intersection<R>(&self, reg: R) -> Region
    where
        R: Into<Region>,
    {
        let reg = reg.into();
        let x0 = reg.x.max(0);
        let x1 = reg.right().min(self.width);
        let w = (x1 - x0).max(0) as u32;
        let y0 = reg.y.max(0);
        let y1 = reg.bottom().min(self.height);
        let h = (y1 - y0).max(0) as u32;
        Region::new(x0, y0, w, h)
    }

    /// Composite a source color to a region of the `Raster`.
    ///
    /// * `reg` Region within `self`.  It can be a `Region` struct, tuple of
    ///         (*x*, *y*, *width*, *height*) or the unit type `()`.  Using
    ///         `()` has the same result as `Raster::region()`.
    /// * `clr` Source `Pixel` color.
    /// * `_op` Compositing operation.
    ///
    /// ### Set entire raster to one color
    /// ```
    /// # use pix::*;
    /// # use pix::ops::Source;
    /// let mut r = Raster::<SRgb32>::with_clear(360, 240);
    /// r.composite_color((), SRgb32::new(0.5, 0.2, 0.8), Source);
    /// ```
    /// ### Set rectangle to solid color
    /// ```
    /// # use pix::*;
    /// # use pix::ops::Source;
    /// let mut r = Raster::<SRgb8>::with_clear(100, 100);
    /// let clr = SRgb8::new(0xDD, 0x96, 0x70);
    /// r.composite_color((20, 40, 25, 50), clr, Source);
    /// ```
    pub fn composite_color<R, O>(&mut self, reg: R, clr: P, _op: O)
    where
        R: Into<Region>,
        O: PorterDuff,
    {
        let reg = self.intersection(reg.into());
        let width = reg.width();
        let height = reg.height();
        if width > 0 && height > 0 {
            let drows = self.rows_mut().skip(reg.y as usize);
            for drow in drows.take(height as usize) {
                let x0 = reg.x as usize;
                let x1 = x0 + width as usize;
                let drow = &mut drow[x0..x1];
                O::composite_color(drow, clr);
            }
        }
    }

    /// Composite from a source `Raster`.
    ///
    /// * `to` Region within `self` (destination).
    /// * `src` Source `Raster`.
    /// * `from` Region within source `Raster`.
    /// * `_op` Compositing operation.
    ///
    /// `to` / `from` can be `Region` structs, tuples of (*x*, *y*, *width*,
    /// *height*) or the unit type `()`.  Using `()` has the same result as
    /// `Raster::region()`.
    ///
    /// ```bob
    /// *------------+      *-------------+
    /// |            |      |    *------+ |
    /// | *------+   |      |    |      | |
    /// | |      |   |      |    | from | |
    /// | |  to  |   | <--- |    +------+ |
    /// | +------+   |      |             |
    /// |            |      |     src     |
    /// |    self    |      +-------------+
    /// +------------+
    /// ```
    /// The composited `Region` is clamped to the smaller of `to` and `from` in
    /// both `X` and `Y` dimensions.  Also, `to` and `from` are clipped to
    /// their respective `Raster` dimensions.
    ///
    /// ### Copy part of one `Raster` to another, converting pixel format
    /// ```
    /// # use pix::*;
    /// # use pix::ops::Source;
    /// let mut r0 = Raster::<SRgb8>::with_clear(100, 100);
    /// let r1 = Raster::with_color(5, 5, SRgb8::new(80, 0, 80));
    /// // ... load image data
    /// r0.composite_raster((40, 40, 5, 5), &r1, (), Source);
    /// ```
    pub fn composite_raster<R0, R1, O>(
        &mut self,
        to: R0,
        src: &Raster<P>,
        from: R1,
        _op: O,
    ) where
        R0: Into<Region>,
        R1: Into<Region>,
        O: PorterDuff,
    {
        let (to, from) = (to.into(), from.into());
        let tx = to.x.min(0).abs();
        let ty = to.y.min(0).abs();
        let fx = from.x.min(0).abs();
        let fy = from.y.min(0).abs();
        let to = self.intersection(to);
        let from = src.intersection(from);
        let width = to.width().min(from.width());
        let height = to.height().min(from.height());
        if width > 0 && height > 0 {
            let to = Region::new(to.x + fx, to.y + fy, width, height);
            let from = Region::new(from.x + tx, from.y + ty, width, height);
            let srows = src.rows().skip(from.y as usize);
            let drows = self.rows_mut().skip(to.y as usize);
            for (drow, srow) in drows.take(height as usize).zip(srows) {
                let x0 = to.x as usize;
                let x1 = x0 + width as usize;
                let drow = &mut drow[x0..x1];
                let srow = &srow[from.x as usize..];
                O::composite(drow, srow);
            }
        }
    }

    /// Get view of pixels as a `u8` slice.
    ///
    /// Q: Is this UB when P::Chan is Ch32?
    pub fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            let (prefix, v, suffix) = &self.pixels.align_to::<u8>();
            debug_assert!(prefix.is_empty());
            debug_assert!(suffix.is_empty());
            v
        }
    }
}

impl<'a, P: Pixel> Rows<'a, P> {
    /// Create a new row `Iterator`.
    fn new(raster: &'a Raster<P>) -> Self {
        let width = usize::try_from(raster.width()).unwrap();
        let chunks = raster.pixels.chunks_exact(width);
        Rows { chunks }
    }
}

impl<'a, P: Pixel> Iterator for Rows<'a, P> {
    type Item = &'a [P];

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next()
    }
}

impl<'a, P: Pixel> RowsMut<'a, P> {
    /// Create a new mutable row `Iterator`.
    fn new(raster: &'a mut Raster<P>) -> Self {
        let width = usize::try_from(raster.width()).unwrap();
        let chunks = raster.pixels.chunks_exact_mut(width);
        RowsMut { chunks }
    }
}

impl<'a, P: Pixel> Iterator for RowsMut<'a, P> {
    type Item = &'a mut [P];

    fn next(&mut self) -> Option<Self::Item> {
        self.chunks.next()
    }
}

impl From<(i32, i32, u32, u32)> for Region {
    fn from(r: (i32, i32, u32, u32)) -> Self {
        Region::new(r.0, r.1, r.2, r.3)
    }
}

impl From<()> for Region {
    fn from(_: ()) -> Self {
        const MAX: u32 = std::i32::MAX as u32;
        Region::new(0, 0, MAX, MAX)
    }
}

impl Region {
    /// Create a new `Region`
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        let width = i32::try_from(width).unwrap_or(0);
        let height = i32::try_from(height).unwrap_or(0);
        Region {
            x,
            y,
            width,
            height,
        }
    }

    /// Get intersection with another `Region`
    pub fn intersection<R>(self, rhs: R) -> Self
    where
        R: Into<Self>,
    {
        let rhs = rhs.into();
        let x0 = self.x.max(rhs.x);
        let x1 = self.right().min(rhs.right());
        let w = (x1 - x0) as u32;
        let y0 = self.y.max(rhs.y);
        let y1 = self.bottom().min(rhs.bottom());
        let h = (y1 - y0) as u32;
        Region::new(x0, y0, w, h)
    }

    /// Get the width
    fn width(self) -> u32 {
        self.width as u32
    }

    /// Get the height
    fn height(self) -> u32 {
        self.height as u32
    }

    /// Get right side
    fn right(self) -> i32 {
        self.x.saturating_add(self.width)
    }

    /// Get bottom side
    fn bottom(self) -> i32 {
        self.y.saturating_add(self.height)
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod test {
    use crate::*;
    use crate::ops::*;
    #[test]
    fn region_size() {
        assert_eq!(std::mem::size_of::<Region>(), 16);
    }
    #[test]
    fn intersect() -> Result<(), ()> {
        let r = Region::new(0, 0, 5, 5);
        assert_eq!(r, Region::new(0, 0, 5, 5));
        assert_eq!(r, r.intersection(Region::new(0, 0, 10, 10)));
        assert_eq!(r, r.intersection(Region::new(-5, -5, 10, 10)));
        assert_eq!(
            Region::new(0, 0, 4, 4),
            r.intersection(Region::new(-1, -1, 5, 5))
        );
        assert_eq!(
            Region::new(1, 2, 1, 3),
            r.intersection(Region::new(1, 2, 1, 100))
        );
        assert_eq!(
            Region::new(2, 1, 3, 1),
            r.intersection(Region::new(2, 1, 100, 1))
        );
        Ok(())
    }
    #[test]
    fn with_buffer_rgb8() {
        let b = vec![
            0xAA,0x00,0x00, 0x00,0x11,0x22, 0x33,0x44,0x55,
            0x00,0xBB,0x00, 0x66,0x77,0x88, 0x99,0xAA,0xBB,
            0x00,0x00,0xCC, 0xCC,0xDD,0xEE, 0xFF,0x00,0x11,
        ];
        let r = Raster::<SRgb8>::with_u8_buffer(3, 3, b);
        let v = vec![
            SRgb8::new(0xAA, 0x00, 0x00), SRgb8::new(0x00, 0x11, 0x22),
            SRgb8::new(0x33, 0x44, 0x55),
            SRgb8::new(0x00, 0xBB, 0x00), SRgb8::new(0x66, 0x77, 0x88),
            SRgb8::new(0x99, 0xAA, 0xBB),
            SRgb8::new(0x00, 0x00, 0xCC), SRgb8::new(0xCC, 0xDD, 0xEE),
            SRgb8::new(0xFF, 0x00, 0x11),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn with_buffer_graya16() {
        let b = vec![
            0x1001,0x5005, 0x1000,0x3002, 0x5004,0x7006,
            0x2002,0x6006, 0x9008,0xB00A, 0xD00C,0xF00E,
            0x3003,0x7007, 0xE00F,0xC00D, 0xA00B,0x8009,
        ];
        let r = Raster::<SGraya16>::with_u16_buffer(3, 3, b);
        let v = vec![
            SGraya16::new(0x1001, 0x5005), SGraya16::new(0x1000, 0x3002),
            SGraya16::new(0x5004, 0x7006),
            SGraya16::new(0x2002, 0x6006), SGraya16::new(0x9008, 0xB00A),
            SGraya16::new(0xD00C, 0xF00E),
            SGraya16::new(0x3003, 0x7007), SGraya16::new(0xE00F, 0xC00D),
            SGraya16::new(0xA00B, 0x8009),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn with_pixels_mask32() {
        let p = vec![
            Mask32::new(0.25), Mask32::new(0.5), Mask32::new(0.75),
            Mask32::new(0.5), Mask32::new(0.6), Mask32::new(0.7),
            Mask32::new(0.85), Mask32::new(0.65), Mask32::new(0.45),
        ];
        let r = Raster::with_pixels(3, 3, p.clone());
        assert_eq!(r.pixels(), &p[..]);
    }
    #[test]
    fn pixel_mut_mask8() {
        let mut r = Raster::<Mask8>::with_clear(3, 3);
        *r.pixel_mut(0, 0) = Mask8::new(0xFF);
        *r.pixel_mut(2, 0) = Mask8::new(0x12);
        *r.pixel_mut(1, 1) = Mask8::new(0x34);
        *r.pixel_mut(0, 2) = Mask8::new(0x56);
        *r.pixel_mut(2, 2) = Mask8::new(0x78);
        let v = vec![
            Mask8::new(0xFF), Mask8::new(0x00), Mask8::new(0x12),
            Mask8::new(0x00), Mask8::new(0x34), Mask8::new(0x00),
            Mask8::new(0x56), Mask8::new(0x00), Mask8::new(0x78),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn pixel_mut_mask16() {
        let mut r = Raster::<Mask16>::with_clear(3, 3);
        *r.pixel_mut(2, 0) = Mask16::new(0x9ABC);
        *r.pixel_mut(1, 1) = Mask16::new(0x5678);
        *r.pixel_mut(0, 2) = Mask16::new(0x1234);
        *r.pixel_mut(0, 0) = Mask16::new(1.0);
        *r.pixel_mut(2, 2) = Mask16::new(0x8080);
        let v = vec![
            Mask16::new(0xFFFF), Mask16::new(0x0000), Mask16::new(0x9ABC),
            Mask16::new(0x0000), Mask16::new(0x5678), Mask16::new(0x0000),
            Mask16::new(0x1234), Mask16::new(0x0000), Mask16::new(0x8080),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn raster_with_color() {
        let r = Raster::<Hwb8>::with_color(3, 3, Hwb8::new(0x80, 0, 0));
        let v = vec![Hwb8::new(0x80, 0, 0); 9];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn composite_color_gray8() {
        let mut r = Raster::<SGray8>::with_clear(3, 3);
        r.composite_color((0, 0, 1, 1), SGray8::new(0x23), Source);
        r.composite_color((10, 10, 1, 1), SGray8::new(0x45), Source);
        r.composite_color((1, 1, 10, 10), SGray8::new(0xBB), Source);
        let v = vec![
            SGray8::new(0x23), SGray8::new(0), SGray8::new(0),
            SGray8::new(0), SGray8::new(0xBB), SGray8::new(0xBB),
            SGray8::new(0), SGray8::new(0xBB), SGray8::new(0xBB),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn composite_color_gray8_over() {
        let clr = Graya8p::new(0x20, 0x40);
        let mut r = Raster::<Graya8p>::with_color(2, 2, clr);
        r.composite_color((0, 0, 3, 1), Graya8p::new(0x60, 0xA0), SourceOver);
        r.composite_color((1, 1, 4, 4), Graya8p::new(0x80, 0x80), SourceOver);
        let v = vec![
            Graya8p::new(0x6B, 0xB7), Graya8p::new(0x6B, 0xB7),
            Graya8p::new(0x20, 0x40), Graya8p::new(0x8F, 0x9F),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn composite_color_srgb8() {
        let mut r = Raster::<SRgb8>::with_clear(3, 3);
        r.composite_color((2, -1, 3, 4), SRgb8::new(0xCC, 0xAA, 0xBB), Source);
        let v = vec![
            SRgb8::new(0, 0, 0), SRgb8::new(0, 0, 0),
            SRgb8::new(0xCC, 0xAA, 0xBB),
            SRgb8::new(0, 0, 0), SRgb8::new(0, 0, 0),
            SRgb8::new(0xCC, 0xAA, 0xBB),
            SRgb8::new(0, 0, 0), SRgb8::new(0, 0, 0),
            SRgb8::new(0xCC, 0xAA, 0xBB),
        ];
        assert_eq!(r.pixels(), &v[..]);
    }
    #[test]
    fn composite_raster_gray() {
        let mut g0 = Raster::<Gray8>::with_clear(3, 3);
        let g1 = Raster::<Gray8>::with_color(3, 3, Gray8::new(0x40));
        let g2 = Raster::<Gray8>::with_color(3, 3, Gray8::new(0x60));
        let g3 = Raster::<Gray8>::with_color(3, 3, Gray8::new(0x80));
        g0.composite_raster((-1, 2, 3, 3), &g1, (), Source);
        g0.composite_raster((2, -1, 3, 3), &g2, (), Source);
        g0.composite_raster((-2, -2, 3, 3), &g3, (), Source);
        let v = vec![
            Gray8::new(0x80), Gray8::new(0x00), Gray8::new(0x60),
            Gray8::new(0x00), Gray8::new(0x00), Gray8::new(0x60),
            Gray8::new(0x40), Gray8::new(0x40), Gray8::new(0x00),
        ];
        assert_eq!(g0.pixels(), &v[..]);
    }
    #[test]
    fn composite_raster_rgb() {
        let mut rgb = Raster::<SRgb8>::with_clear(3, 3);
        let gray = Raster::<SGray16>::with_color(3, 3, SGray16::new(0x8000));
        let r = Raster::with_raster(&gray);
        rgb.composite_raster((), &r, (0, 1, 3, 3), Source);
        let mut v = vec![SRgb8::new(0x80, 0x80, 0x80); 6];
        v.extend_from_slice(&vec![SRgb8::new(0, 0, 0); 3]);
        assert_eq!(rgb.pixels(), &v[..]);
    }
    #[test]
    fn with_raster_rgb() {
        let r = Raster::<SRgb8>::with_clear(50, 50);
        let _ = Raster::<SRgb16>::with_raster(&r);
        let _ = Raster::<SRgb32>::with_raster(&r);
        let _ = Raster::<SRgba8>::with_raster(&r);
        let _ = Raster::<SRgba16>::with_raster(&r);
        let _ = Raster::<SRgba32>::with_raster(&r);
        let _ = Raster::<SGray8>::with_raster(&r);
        let _ = Raster::<SGray16>::with_raster(&r);
        let _ = Raster::<SGray32>::with_raster(&r);
        let _ = Raster::<SGraya8>::with_raster(&r);
        let _ = Raster::<SGraya16>::with_raster(&r);
        let _ = Raster::<SGraya32>::with_raster(&r);
        let _ = Raster::<Mask8>::with_raster(&r);
        let _ = Raster::<Mask16>::with_raster(&r);
        let _ = Raster::<Mask32>::with_raster(&r);
    }
    #[test]
    fn with_raster_mask8() {
        let r = Raster::<Mask8>::with_clear(50, 50);
        let _ = Raster::<SRgb8>::with_raster(&r);
        let _ = Raster::<SRgb16>::with_raster(&r);
        let _ = Raster::<SRgb32>::with_raster(&r);
        let _ = Raster::<SRgba8>::with_raster(&r);
        let _ = Raster::<SRgba16>::with_raster(&r);
        let _ = Raster::<SRgba32>::with_raster(&r);
        let _ = Raster::<SGray8>::with_raster(&r);
        let _ = Raster::<SGray16>::with_raster(&r);
        let _ = Raster::<SGray32>::with_raster(&r);
        let _ = Raster::<SGraya8>::with_raster(&r);
        let _ = Raster::<SGraya16>::with_raster(&r);
        let _ = Raster::<SGraya32>::with_raster(&r);
        let _ = Raster::<Mask8>::with_raster(&r);
        let _ = Raster::<Mask16>::with_raster(&r);
        let _ = Raster::<Mask32>::with_raster(&r);
    }
}
