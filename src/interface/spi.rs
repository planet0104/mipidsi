use embedded_hal::{digital::OutputPin, spi::SpiDevice};

use super::Interface;

/// Spi interface error
#[derive(Clone, Copy, Debug)]
pub enum SpiError<SPI, DC> {
    /// SPI bus error
    Spi(SPI),
    /// Data/command pin error
    Dc(DC),
}

/// Spi interface, including a buffer
///
/// The buffer is used to gather batches of pixel data to be sent over SPI.
/// Larger buffers will genererally be faster (with diminishing returns), at the expense of using more RAM.
/// The buffer should be at least big enough to hold a few pixels of data.
///
/// You may want to use [static_cell](https://crates.io/crates/static_cell)
/// to obtain a `&'static mut [u8; N]` buffer.
pub struct SpiInterface<'a, SPI, DC> {
    spi: SPI,
    dc: DC,
    buffer: &'a mut [u8],
}

impl<'a, SPI: SpiDevice, DC: OutputPin> SpiInterface<'a, SPI, DC> {
    /// Create new interface
    pub fn new(spi: SPI, dc: DC, buffer: &'a mut [u8]) -> Self {
        Self { spi, dc, buffer }
    }
}

impl<SPI: SpiDevice, DC: OutputPin> Interface for SpiInterface<'_, SPI, DC> {
    type Word = u8;
    type Error = SpiError<SPI::Error, DC::Error>;

    fn send_command(&mut self, command: u8, args: &[u8]) -> Result<(), Self::Error> {
        self.dc.set_low().map_err(SpiError::Dc)?;
        self.spi.write(&[command]).map_err(SpiError::Spi)?;
        self.dc.set_high().map_err(SpiError::Dc)?;
        self.spi.write(args).map_err(SpiError::Spi)?;
        Ok(())
    }

    fn send_pixels<const N: usize>(
        &mut self,
        pixels: impl IntoIterator<Item = [Self::Word; N]>,
    ) -> Result<(), Self::Error> {
        let mut arrays = pixels.into_iter();

        assert!(self.buffer.len() >= N);

        let mut done = false;
        while !done {
            let mut i = 0;
            for chunk in self.buffer.chunks_exact_mut(N) {
                if let Some(array) = arrays.next() {
                    let chunk: &mut [u8; N] = chunk.try_into().unwrap();
                    *chunk = array;
                    i += N;
                } else {
                    done = true;
                    break;
                };
            }
            self.spi.write(&self.buffer[..i]).map_err(SpiError::Spi)?;
        }
        Ok(())
    }
    
    fn send_pixels_buffer(
        &mut self,
        pixels: &[u8],
    ) -> Result<(), Self::Error> {
        self.spi.write(pixels).map_err(SpiError::Spi)?;
        Ok(())
    }

    fn send_pixels_buffer_u16(
        &mut self,
        pixels: &[u16],
    ) -> Result<(), Self::Error> {
        use byte_slice_cast::*;
        self.spi.write(pixels.as_byte_slice()).map_err(SpiError::Spi)?;
        Ok(())
    }

    fn send_repeated_pixel<const N: usize>(
        &mut self,
        pixel: [Self::Word; N],
        count: u32,
    ) -> Result<(), Self::Error> {
        let fill_count = core::cmp::min(count, (self.buffer.len() / N) as u32);
        let filled_len = fill_count as usize * N;
        for chunk in self.buffer[..(filled_len)].chunks_exact_mut(N) {
            let chunk: &mut [u8; N] = chunk.try_into().unwrap();
            *chunk = pixel;
        }

        let mut count = count;
        while count >= fill_count {
            self.spi
                .write(&self.buffer[..filled_len])
                .map_err(SpiError::Spi)?;
            count -= fill_count;
        }
        if count != 0 {
            self.spi
                .write(&self.buffer[..(count as usize * pixel.len())])
                .map_err(SpiError::Spi)?;
        }
        Ok(())
    }
}
