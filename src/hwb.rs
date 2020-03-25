// hwb.rs       HWB color model
//
// Copyright (c) 2020  Douglas P Lau
//
use crate::alpha::{
    self, AChannel, Opaque, Premultiplied, Straight, Translucent,
};
use crate::gamma::{self, Linear};
use crate::hue::{Hexcone, rgb_to_hue_chroma_value};
use crate::model::Channels;
use crate::{Ch16, Ch32, Ch8, Channel, ColorModel, Pixel};
use std::any::TypeId;
use std::marker::PhantomData;

/// `HWB` [color model].
///
/// The components are *hue*, *whiteness* and *blackness*, with optional
/// *[alpha]*.
///
/// [alpha]: alpha/trait.AChannel.html
/// [color model]: trait.ColorModel.html
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct Hwb<C, A, M, G>
where
    C: Channel,
    A: AChannel<Chan = C>,
    M: alpha::Mode,
    G: gamma::Mode,
{
    hue: C,
    whiteness: C,
    blackness: C,
    alpha: A,
    mode: PhantomData<M>,
    gamma: PhantomData<G>,
}

impl<C, A, M, G> Hwb<C, A, M, G>
where
    C: Channel,
    A: AChannel<Chan = C> + From<C>,
    M: alpha::Mode,
    G: gamma::Mode,
{
    /// Create an `Hwb` color.
    ///
    /// ## Example
    /// ```
    /// # use pix::*;
    /// let opaque_hwb = Hwb8::new(50, 64, 0, ());
    /// let translucent_hwb = Hwba8::new(100, 0, 128, 200);
    /// ```
    pub fn new<H, B>(hue: H, whiteness: H, blackness: H, alpha: B) -> Self
    where
        C: From<H>,
        A: From<B>,
    {
        let hue = C::from(hue);
        let whiteness = C::from(whiteness);
        let blackness = C::from(blackness);
        let alpha = A::from(alpha);
        Hwb {
            hue,
            whiteness,
            blackness,
            alpha,
            mode: PhantomData,
            gamma: PhantomData,
        }
    }

    /// Get the *hue* component.
    pub fn hue(self) -> C {
        self.hue
    }

    /// Get the *whiteness* component.
    pub fn whiteness(self) -> C {
        self.whiteness
    }

    /// Get the *blackness* component.
    pub fn blackness(self) -> C {
        self.blackness
    }

    /// Get *whiteness* and *blackness* clamped to 1.0 at the same ratio
    fn whiteness_blackness(self) -> (C, C) {
        let whiteness = self.whiteness();
        let blackness = self.blackness();
        if whiteness + blackness - blackness < whiteness {
            let (w, b) = (whiteness.into(), blackness.into());
            let ratio = 1.0 / (w + b);
            (C::from(w * ratio), C::from(b * ratio))
        } else {
            (whiteness, blackness)
        }
    }

    /// Convert into *red*, *green*, *blue* and *alpha* components
    fn into_rgba(self) -> [C; 4] {
        let (whiteness, blackness) = self.whiteness_blackness();
        let v = C::MAX - blackness;
        let chroma = v - whiteness;
        let hp = self.hue().into() * 6.0; // 0.0..=6.0
        let hc = Hexcone::from_hue_prime(hp);
        let (red, green, blue) = hc.rgb(chroma);
        let m = v - chroma;
        [red + m, green + m, blue + m, self.alpha()]
    }

    /// Convert from *red*, *green*, *blue* and *alpha* components
    fn from_rgba(rgba: [C; 4]) -> Self {
        let red = rgba[0];
        let green = rgba[1];
        let blue = rgba[2];
        let alpha = rgba[3];
        let (hue, chroma, val) = rgb_to_hue_chroma_value(red, green, blue);
        let sat_v = if val > C::MIN { chroma / val } else { C::MIN };
        let whiteness = (C::MAX - sat_v) * val;
        let blackness = C::MAX - val;
        Hwb::new(hue, whiteness, blackness, alpha)
    }
}

impl<C, A, M, G> ColorModel for Hwb<C, A, M, G>
where
    C: Channel,
    A: AChannel<Chan = C> + From<C>,
    M: alpha::Mode,
    G: gamma::Mode,
{
    type Chan = C;

    /// Get the *alpha* component
    fn alpha(self) -> Self::Chan {
        self.alpha.value()
    }

    /// Convert into channels shared by types
    fn into_channels<R: ColorModel>(self) -> Channels<C> {
        if TypeId::of::<Self>() == TypeId::of::<R>() {
            Channels::new([
                self.whiteness(),
                self.blackness(),
                self.alpha(),
                self.hue(),
            ], 2)
        } else {
            Channels::new(self.into_rgba(), 3)
        }
    }

    /// Convert from channels shared by types
    fn from_channels<R: ColorModel>(channels: Channels<C>) -> Self {
        if TypeId::of::<Self>() == TypeId::of::<R>() {
            debug_assert_eq!(channels.alpha(), 2);
            let ch = channels.into_array();
            let whiteness = ch[0];
            let blackness = ch[1];
            let alpha = ch[2];
            let hue = ch[3];
            Hwb::new(hue, whiteness, blackness, alpha)
        } else {
            debug_assert_eq!(channels.alpha(), 3);
            Self::from_rgba(channels.into_array())
        }
    }
}

impl<C, A, M, G> Pixel for Hwb<C, A, M, G>
where
    C: Channel,
    A: AChannel<Chan = C> + From<C>,
    M: alpha::Mode,
    G: gamma::Mode,
{
    type Alpha = M;
    type Gamma = G;
}

impl<C, A, M, G> Iterator for Hwb<C, A, M, G>
where
    C: Channel,
    A: AChannel<Chan = C>,
    M: alpha::Mode,
    G: gamma::Mode,
{
    type Item = Self;

    fn next(&mut self) -> Option<Self::Item> {
        Some(*self)
    }
}

/// [Hwb](struct.Hwb.html) 8-bit [opaque](alpha/struct.Opaque.html) (no alpha)
/// [linear](gamma/struct.Linear.html) gamma [pixel](trait.Pixel.html) format.
pub type Hwb8 = Hwb<Ch8, Opaque<Ch8>, Straight, Linear>;
/// [Hwb](struct.Hwb.html) 16-bit [opaque](alpha/struct.Opaque.html) (no alpha)
/// [linear](gamma/struct.Linear.html) gamma [pixel](trait.Pixel.html) format.
pub type Hwb16 = Hwb<Ch16, Opaque<Ch16>, Straight, Linear>;
/// [Hwb](struct.Hwb.html) 32-bit [opaque](alpha/struct.Opaque.html) (no alpha)
/// [linear](gamma/struct.Linear.html) gamma [pixel](trait.Pixel.html) format.
pub type Hwb32 = Hwb<Ch32, Opaque<Ch32>, Straight, Linear>;

/// [Hwb](struct.Hwb.html) 8-bit [straight](alpha/struct.Straight.html) alpha
/// [linear](gamma/struct.Linear.html) gamma [pixel](trait.Pixel.html) format.
pub type Hwba8 = Hwb<Ch8, Translucent<Ch8>, Straight, Linear>;
/// [Hwb](struct.Hwb.html) 16-bit [straight](alpha/struct.Straight.html) alpha
/// [linear](gamma/struct.Linear.html) gamma [pixel](trait.Pixel.html) format.
pub type Hwba16 = Hwb<Ch16, Translucent<Ch16>, Straight, Linear>;
/// [Hwb](struct.Hwb.html) 32-bit [straight](alpha/struct.Straight.html) alpha
/// [linear](gamma/struct.Linear.html) gamma [pixel](trait.Pixel.html) format.
pub type Hwba32 = Hwb<Ch32, Translucent<Ch32>, Straight, Linear>;

/// [Hwb](struct.Hwb.html) 8-bit
/// [premultiplied](alpha/struct.Premultiplied.html) alpha
/// [linear](gamma/struct.Linear.html) gamma [pixel](trait.Pixel.html) format.
pub type Hwba8p = Hwb<Ch8, Translucent<Ch8>, Premultiplied, Linear>;
/// [Hwb](struct.Hwb.html) 16-bit
/// [premultiplied](alpha/struct.Premultiplied.html) alpha
/// [linear](gamma/struct.Linear.html) gamma [pixel](trait.Pixel.html) format.
pub type Hwba16p = Hwb<Ch16, Translucent<Ch16>, Premultiplied, Linear>;
/// [Hwb](struct.Hwb.html) 32-bit
/// [premultiplied](alpha/struct.Premultiplied.html) alpha
/// [linear](gamma/struct.Linear.html) gamma [pixel](trait.Pixel.html) format.
pub type Hwba32p = Hwb<Ch32, Translucent<Ch32>, Premultiplied, Linear>;

#[cfg(test)]
mod test {
    use super::super::*;
    use super::*;

    #[test]
    fn check_sizes() {
        assert_eq!(std::mem::size_of::<Hwb8>(), 3);
        assert_eq!(std::mem::size_of::<Hwb16>(), 6);
        assert_eq!(std::mem::size_of::<Hwb32>(), 12);
        assert_eq!(std::mem::size_of::<Hwba8>(), 4);
        assert_eq!(std::mem::size_of::<Hwba16>(), 8);
        assert_eq!(std::mem::size_of::<Hwba32>(), 16);
    }

    #[test]
    fn hwb_to_rgb() {
        assert_eq!(
            Rgb8::new(127, 127, 127, ()),
            Hwb8::new(0, 128, 128, ()).convert(),
        );
        assert_eq!(
            Rgb8::new(127, 127, 127, ()),
            Hwb8::new(0, 255, 255, ()).convert(),
        );
        assert_eq!(
            Rgb8::new(85, 85, 85, ()),
            Hwb8::new(0, 128, 255, ()).convert(),
        );
        assert_eq!(
            Rgb8::new(255, 0, 0, ()),
            Hwb8::new(0, 0, 0, ()).convert(),
        );
        assert_eq!(
            Rgb8::new(255, 255, 128, ()),
            Hwb32::new(60.0 / 360.0, 0.5, 0.0, ()).convert(),
        );
        assert_eq!(
            Rgb8::new(0, 127, 0, ()),
            Hwb16::new(21845, 0, 32768, ()).convert(),
        );
        assert_eq!(
            Rgb8::new(128, 255, 255, ()),
            Hwb32::new(0.5, 0.5, 0.0, ()).convert(),
        );
        assert_eq!(
            Rgb8::new(0, 0, 128, ()),
            Hwb32::new(240.0 / 360.0, 0.0, 0.5, ()).convert(),
        );
        assert_eq!(
            Rgb8::new(255, 128, 255, ()),
            Hwb32::new(300.0 / 360.0, 0.5, 0.0, ()).convert(),
        );
    }

    #[test]
    fn rgb_to_hwb() {
        assert_eq!(
            Hwb8::new(0, 0, 0, ()),
            Rgb8::new(255, 0, 0, ()).convert(),
        );
        assert_eq!(
            Hwb8::new(0, 64, 0, ()),
            Rgb8::new(255, 64, 64, ()).convert(),
        );
        assert_eq!(
            Hwb32::new(60.0 / 360.0, 0.0, 0.50196075, ()),
            Rgb8::new(127, 127, 0, ()).convert(),
        );
        assert_eq!(
            Hwb16::new(21845, 8224, 0, ()),
            Rgb8::new(32, 255, 32, ()).convert(),
        );
        assert_eq!(
            Hwb32::new(0.5, 0.0, 0.7490196, ()),
            Rgb8::new(0, 64, 64, ()).convert(),
        );
        assert_eq!(
            Hwb32::new(240.0 / 360.0, 0.7529412, 0.0, ()),
            Rgb8::new(192, 192, 255, ()).convert(),
        );
        assert_eq!(
            Hwb32::new(300.0 / 360.0, 0.0, 0.0, ()),
            Rgb8::new(255, 0, 255, ()).convert(),
        );
    }
}
