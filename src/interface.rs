//! Interface traits and implementations

mod spi;
use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666, RgbColor};
pub use spi::*;

mod parallel;
pub use parallel::*;

/// Command and pixel interface
pub trait Interface {
    /// The native width of the interface
    ///
    /// In most cases this will be u8, except for larger parallel interfaces such as
    /// 16 bit (currently supported)
    /// or 9 or 18 bit (currently unsupported)
    type Word: Copy;

    /// Error type
    type Error: core::fmt::Debug;

    /// Send a command with optional parameters
    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error>;

    /// Send a sequence of pixels
    ///
    /// `WriteMemoryStart` must be sent before calling this function
    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::Word; N]>,
    ) -> Result<(), Self::Error>;

    /// Send a sequence of pixels
    ///
    /// `WriteMemoryStart` must be sent before calling this function
    fn send_pixels_buffer(
        &mut self,
        pixels: &[u8],
    ) -> Result<(), Self::Error>;

    /// Send a sequence of pixels
    ///
    /// `WriteMemoryStart` must be sent before calling this function
    fn send_pixels_buffer_u16(
        &mut self,
        pixels: &[u16],
    ) -> Result<(), Self::Error>;

    /// Send the same pixel value multiple times
    ///
    /// `WriteMemoryStart` must be sent before calling this function
    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [Self::Word; N],
        count: u32,
    ) -> Result<(), Self::Error>;
}

impl<T: Interface> Interface for &mut T {
    type Word = T::Word;
    type Error = T::Error;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        T::send_command(self, command, args)
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::Word; N]>,
    ) -> Result<(), Self::Error> {
        T::send_pixels(self, pixels)
    }

    fn send_pixels_buffer(
        &mut self,
        pixels: &[u8],
    ) -> Result<(), Self::Error> {
        T::send_pixels_buffer(self, pixels)
    }

    fn send_pixels_buffer_u16(
        &mut self,
        pixels: &[u16],
    ) -> Result<(), Self::Error> {
        T::send_pixels_buffer_u16(self, pixels)
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [Self::Word; N],
        count: u32,
    ) -> Result<(), Self::Error> {
        T::send_repeated_pixel(self, pixel, count)
    }
}

fn rgb565_to_bytes(pixel: Rgb565) -> [u8; 2] {
    embedded_graphics_core::pixelcolor::raw::ToBytes::to_be_bytes(pixel)
}
fn rgb565_to_u16(pixel: Rgb565) -> [u16; 1] {
    [u16::from_ne_bytes(
        embedded_graphics_core::pixelcolor::raw::ToBytes::to_ne_bytes(pixel),
    )]
}
fn rgb666_to_bytes(pixel: Rgb666) -> [u8; 3] {
    [pixel.r(), pixel.g(), pixel.b()].map(|x| x << 2)
}

/// This is an implementation detail, it should not be implemented or used outside this crate
pub trait InterfacePixelFormat<Word> {
    // this should just be
    // const N: usize;
    // fn convert(self) -> [Word; Self::N];
    // but that doesn't work yet

    #[doc(hidden)]
    fn send_pixels<DI: Interface<Word = Word>>(
        di: &mut DI,
        pixels: impl IntoIterator<Item = Self>,
    ) -> Result<(), DI::Error>;

    #[doc(hidden)]
    fn send_pixels_buffer<DI: Interface<Word = Word>>(
        di: &mut DI,
        pixels: &[u8],
    ) -> Result<(), DI::Error>;

    #[doc(hidden)]
    fn send_pixels_buffer_u16<DI: Interface<Word = Word>>(
        di: &mut DI,
        pixels: &[u16],
    ) -> Result<(), DI::Error>;

    #[doc(hidden)]
    fn send_repeated_pixel<DI: Interface<Word = Word>>(
        di: &mut DI,
        pixel: Self,
        count: u32,
    ) -> Result<(), DI::Error>;
}

impl InterfacePixelFormat<u8> for Rgb565 {
    fn send_pixels<DI: Interface<Word = u8>>(
        di: &mut DI,
        pixels: impl IntoIterator<Item = Self>,
    ) -> Result<(), DI::Error> {
        di.send_pixels(pixels.into_iter().map(rgb565_to_bytes))
    }

    fn send_pixels_buffer<DI: Interface<Word = u8>>(
        di: &mut DI,
        pixels: &[u8],
    ) -> Result<(), DI::Error> {
        di.send_pixels_buffer(pixels)
    }

    fn send_pixels_buffer_u16<DI: Interface<Word = u8>>(
        di: &mut DI,
        pixels: &[u16],
    ) -> Result<(), DI::Error> {
        di.send_pixels_buffer_u16(pixels)
    }

    fn send_repeated_pixel<DI: Interface<Word = u8>>(
        di: &mut DI,
        pixel: Self,
        count: u32,
    ) -> Result<(), DI::Error> {
        di.send_repeated_pixel(rgb565_to_bytes(pixel), count)
    }
}

impl InterfacePixelFormat<u8> for Rgb666 {
    fn send_pixels<DI: Interface<Word = u8>>(
        di: &mut DI,
        pixels: impl IntoIterator<Item = Self>,
    ) -> Result<(), DI::Error> {
        di.send_pixels(pixels.into_iter().map(rgb666_to_bytes))
    }

    fn send_pixels_buffer<DI: Interface<Word = u8>>(
        di: &mut DI,
        pixels: &[u8],
    ) -> Result<(), DI::Error> {
        di.send_pixels_buffer(pixels)
    }

    fn send_pixels_buffer_u16<DI: Interface<Word = u8>>(
        di: &mut DI,
        pixels: &[u16],
    ) -> Result<(), DI::Error> {
        di.send_pixels_buffer_u16(pixels)
    }

    fn send_repeated_pixel<DI: Interface<Word = u8>>(
        di: &mut DI,
        pixel: Self,
        count: u32,
    ) -> Result<(), DI::Error> {
        di.send_repeated_pixel(rgb666_to_bytes(pixel), count)
    }
}

impl InterfacePixelFormat<u16> for Rgb565 {
    fn send_pixels<DI: Interface<Word = u16>>(
        di: &mut DI,
        pixels: impl IntoIterator<Item = Self>,
    ) -> Result<(), DI::Error> {
        di.send_pixels(pixels.into_iter().map(rgb565_to_u16))
    }

    fn send_pixels_buffer<DI: Interface<Word = u16>>(
        di: &mut DI,
        pixels: &[u8],
    ) -> Result<(), DI::Error> {
        di.send_pixels_buffer(pixels)
    }

    fn send_pixels_buffer_u16<DI: Interface<Word = u16>>(
        di: &mut DI,
        pixels: &[u16],
    ) -> Result<(), DI::Error> {
        di.send_pixels_buffer_u16(pixels)
    }

    fn send_repeated_pixel<DI: Interface<Word = u16>>(
        di: &mut DI,
        pixel: Self,
        count: u32,
    ) -> Result<(), DI::Error> {
        di.send_repeated_pixel(rgb565_to_u16(pixel), count)
    }
}
