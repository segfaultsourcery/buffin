use std::any::type_name;

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
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, pos: 0 }
    }

    pub fn new_filled(buffer: &'a mut [u8]) -> Self {
        let len = buffer.len();
        Self { buffer, pos: len }
    }

    pub fn with_pos(buffer: &'a mut [u8], pos: usize) -> Self {
        Self { buffer, pos }
    }

    pub fn len(&self) -> usize {
        self.pos
    }

    pub fn bytes(&'a self) -> &'a [u8] {
        &self.buffer[..self.pos]
    }

    pub fn clear(&mut self) {
        self.pos = 0;
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        if self.pos + bytes.len() >= self.buffer.len() {
            bail!("Buffer is too small");
        }

        self.buffer[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();

        Ok(())
    }

    pub fn add<T: ToBytes>(&mut self, b: &T) -> Result<()> {
        self.pos += b.to_bytes(&mut self.buffer[self.pos..])?;
        Ok(())
    }

    /// Remove the n first values.
    pub fn remove_first(&mut self, n: usize) {
        self.buffer.copy_within(n..self.pos, 0);
        self.pos -= n;
    }

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
