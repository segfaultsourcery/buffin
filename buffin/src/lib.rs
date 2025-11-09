#![cfg_attr(feature = "no_std", no_std)]

use core::any::type_name;
use eyre::{Result, bail};
use nom::IResult;
use tracing::warn;

pub mod basic_types;

pub struct Buffin<'a> {
    buffer: &'a mut [u8],
    pos: usize,
}

#[allow(clippy::result_unit_err)]
impl<'a> Buffin<'a> {
    /// Create a new instance with an empty buffer.
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, pos: 0 }
    }

    /// Create a new instance that considers itself filled by the provided buffer.
    pub fn new_filled(buffer: &'a mut [u8]) -> Self {
        let len = buffer.len();
        Self { buffer, pos: len }
    }

    /// Create a new instance that considers the first `pos` bytes as data.
    pub fn with_pos(buffer: &'a mut [u8], pos: usize) -> Self {
        Self { buffer, pos }
    }

    /// Returns the number of used bytes.
    pub fn len(&self) -> usize {
        self.pos
    }

    /// Returns the used bytes as a slice.
    pub fn bytes(&'a self) -> &'a [u8] {
        &self.buffer[..self.pos]
    }

    /// Empty the buffer.
    pub fn clear(&mut self) {
        self.pos = 0;
    }

    /// Returns true if the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Adds the given bytes as is.
    pub fn add_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        if self.pos + bytes.len() >= self.buffer.len() {
            bail!("Buffer is too small");
        }

        self.buffer[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();

        Ok(())
    }

    /// Adds something that implements ToBytes.
    pub fn add<T: ToBytes>(&mut self, b: &T) -> Result<()> {
        self.pos += b.to_bytes(&mut self.buffer[self.pos..])?;
        Ok(())
    }

    /// Remove the n first bytes.
    pub fn remove_first(&mut self, n: usize) {
        self.buffer.copy_within(n..self.pos, 0);
        self.pos -= n;
    }

    /// Attempts to pop the first item of the given type.
    pub fn pop<T: FromBytes>(&mut self) -> Result<T, PopFailure> {
        match T::from_bytes(self.bytes()) {
            Ok((remainder, result)) => {
                let removed = self.len() - remainder.len();
                self.remove_first(removed);
                Ok(result)
            }
            Result::Err(err) => {
                if err.is_incomplete() {
                    Err(PopFailure::Incomplete)
                } else {
                    warn!(?err, type=?type_name::<T>(), bytes_left=?self.len(), "failed to parse");
                    Err(PopFailure::Invalid)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum PopFailure {
    Invalid,
    Incomplete,
}

pub trait ToBytes: Sized {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize>;
}

pub trait FromBytes: Sized {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self>;
}
