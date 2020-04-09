// matte.rs     Alpha matte color model.
//
// Copyright (c) 2019-2020  Douglas P Lau
// Copyright (c) 2019-2020  Jeron Aldaron Lau
//
use crate::chan::{Ch16, Ch32, Ch8, Channel, Linear, Straight};
use crate::clr::ColorModel;
use crate::el::{Pix1, PixRgba, Pixel};
use std::ops::Range;

/// Matte [color model].
///
/// The component is *[alpha]* only.
///
/// [alpha]: #method.alpha
/// [color model]: trait.ColorModel.html
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Matte {}

impl Matte {
    /// Get the *alpha* component.
    ///
    /// # Example: Matte Alpha
    /// ```
    /// use pix::Matte8;
    /// use pix::chan::Ch8;
    /// use pix::clr::Matte;
    ///
    /// let p = Matte8::new(0x94);
    /// assert_eq!(Matte::alpha(p), Ch8::new(0x94));
    /// ```
    pub fn alpha<P: Pixel>(p: P) -> P::Chan
    where
        P: Pixel<Model = Self>,
    {
        p.one()
    }
}

impl ColorModel for Matte {
    const CIRCULAR: Range<usize> = 0..0;
    const LINEAR: Range<usize> = 0..0;
    const ALPHA: usize = 0;

    /// Convert into *red*, *green*, *blue* and *alpha* components
    fn into_rgba<P>(p: P) -> PixRgba<P>
    where
        P: Pixel<Model = Self>,
    {
        let max = P::Chan::MAX.into();
        PixRgba::<P>::new(max, max, max, Self::alpha(p).into())
    }

    /// Convert from *red*, *green*, *blue* and *alpha* components
    fn from_rgba<P>(rgba: PixRgba<P>) -> P
    where
        P: Pixel<Model = Self>,
    {
        let chan = rgba.channels();
        P::from_channels(&[chan[3]])
    }
}

/// [Matte](clr/struct.Matte.html) 8-bit [straight](chan/struct.Straight.html)
/// alpha [linear](chan/struct.Linear.html) gamma [pixel](el/trait.Pixel.html)
/// format.
pub type Matte8 = Pix1<Ch8, Matte, Straight, Linear>;

/// [Matte](clr/struct.Matte.html) 16-bit [straight](chan/struct.Straight.html)
/// alpha [linear](chan/struct.Linear.html) gamma [pixel](el/trait.Pixel.html)
/// format.
pub type Matte16 = Pix1<Ch16, Matte, Straight, Linear>;

/// [Matte](clr/struct.Matte.html) 32-bit [straight](chan/struct.Straight.html)
/// alpha [linear](chan/struct.Linear.html) gamma [pixel](el/trait.Pixel.html)
/// format.
pub type Matte32 = Pix1<Ch32, Matte, Straight, Linear>;