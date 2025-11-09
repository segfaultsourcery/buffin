use crate::{Buffin, FromBytes, ToBytes};
use eyre::Result;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::tag,
    combinator::map,
    number::streaming::{le_u8, le_u16, le_u32, le_u64},
};

#[cfg(not(feature = "no_std"))]
use eyre::bail;

#[cfg(not(feature = "no_std"))]
use nom::{
    bytes::streaming::take,
    error::{Error, ErrorKind},
};

#[cfg(not(feature = "no_std"))]
use std::{ops::RangeInclusive, path::PathBuf};

#[cfg(feature = "no_std")]
use core::ops::RangeInclusive;

#[cfg(not(feature = "no_std"))]
impl ToBytes for String {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);

        buffer.add(&(self.bytes().len() as u32))?;
        buffer.add_bytes(self.as_bytes())?;

        Ok(buffer.len())
    }
}

#[cfg(not(feature = "no_std"))]
impl ToBytes for &String {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);

        buffer.add(&(self.bytes().len() as u32))?;
        buffer.add_bytes(self.as_bytes())?;

        Ok(buffer.len())
    }
}

#[cfg(not(feature = "no_std"))]
impl FromBytes for String {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        let (buffer, len) = le_u32(buffer)?;
        let (buffer, bytes) = take(len)(buffer)?;
        match String::from_utf8(bytes.to_vec()) {
            Ok(s) => Ok((buffer, s)),
            Err(_) => IResult::Err(nom::Err::Failure(Error {
                input: buffer,
                code: ErrorKind::Fail,
            })),
        }
    }
}

impl ToBytes for u32 {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);
        buffer.add_bytes(&self.to_le_bytes())?;
        Ok(buffer.len())
    }
}

impl FromBytes for u32 {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        le_u32(buffer)
    }
}

impl ToBytes for u64 {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);
        buffer.add_bytes(&self.to_le_bytes())?;
        Ok(buffer.len())
    }
}

impl FromBytes for u64 {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        le_u64(buffer)
    }
}

impl ToBytes for u16 {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);
        buffer.add_bytes(&self.to_le_bytes())?;
        Ok(buffer.len())
    }
}

impl FromBytes for u16 {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        le_u16(buffer)
    }
}

impl ToBytes for u8 {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);
        buffer.add_bytes(&self.to_le_bytes())?;
        Ok(buffer.len())
    }
}

impl FromBytes for u8 {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        le_u8(buffer)
    }
}

//
// ToBytes and FromBytes for slices and vectors. Generalized.
//
impl<T> ToBytes for &[T]
where
    T: ToBytes,
{
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);

        buffer.add(&(self.len() as u32))?;
        for item in self.iter() {
            buffer.add(item)?;
        }

        Ok(buffer.len())
    }
}

#[cfg(not(feature = "no_std"))]
impl<T> ToBytes for Vec<T>
where
    T: ToBytes,
{
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        self.as_slice().to_bytes(buffer)
    }
}

#[cfg(not(feature = "no_std"))]
impl<T> ToBytes for &Vec<T>
where
    T: ToBytes,
{
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        self.as_slice().to_bytes(buffer)
    }
}

#[cfg(not(feature = "no_std"))]
impl<T> FromBytes for Vec<T>
where
    T: FromBytes,
{
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        let (buffer, len) = le_u32(buffer)?;

        let mut buffer = buffer;
        let mut result = vec![];

        for _ in 0..len {
            let (b, it) = T::from_bytes(&buffer)?;
            result.push(it);
            buffer = b;
        }

        Ok((buffer, result))
    }
}

impl<T: ToBytes> ToBytes for RangeInclusive<T> {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);

        buffer.add(self.start())?;
        buffer.add(self.end())?;

        Ok(buffer.len())
    }
}

impl<T: FromBytes> FromBytes for RangeInclusive<T> {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        let (buffer, start) = T::from_bytes(buffer)?;
        let (buffer, end) = T::from_bytes(buffer)?;
        Ok((buffer, RangeInclusive::new(start, end)))
    }
}

impl<T: ToBytes> ToBytes for Option<T> {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);

        match self {
            Some(item) => {
                buffer.add_bytes(b"+")?;
                buffer.add(item)?;
            }
            None => {
                buffer.add_bytes(b"-")?;
            }
        }

        Ok(buffer.len())
    }
}

impl<T: FromBytes> FromBytes for Option<T> {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        alt((
            map((tag("+"), T::from_bytes), |(_, item)| Some(item)),
            map(tag("-"), |_| None),
        ))
        .parse(buffer)
    }
}

#[cfg(not(feature = "no_std"))]
impl ToBytes for PathBuf {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        let mut buffer = Buffin::new(buffer);

        let path = match self.to_str() {
            Some(p) => p.to_string(),
            None => bail!("invalid path"),
        };

        buffer.add(&path)?;
        Ok(buffer.len())
    }
}

#[cfg(not(feature = "no_std"))]
impl FromBytes for PathBuf {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        let (buffer, path) = String::from_bytes(buffer)?;
        Ok((buffer, PathBuf::from(path)))
    }
}
