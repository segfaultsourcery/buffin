_This crate is best used together with the [buffin_derive](https://crates.io/crates/buffin_derive) crate._

# Buffin

This crate allows you to do quick and easy (de)serialization of data, originally meant for passing data to and from small embedded applications.

Best used together with [buffin_derive](https://crates.io/crates/buffin_derive).

## Examples

### Using buffin_derive

Here's a simple enum:

```rust
use buffin::Buffin;
use buffin_derive::{FromBytes, ToBytes};

#[derive(ToBytes, FromBytes)]
enum Message {
    #[tag("j")]
    Join {
        channel: String,
    },

    #[tag("l")]
    Leave {
        channel: String,
    },

    // No tag here, see below.
    Say {
        channel: String,
        message: String,
    },
}
```

Here's what it'll serialize to:

```
Message::Join { channel: "mychannel".to_string() }

becomes

j 09 00 00 00 m y c h a n n e l
^ ^           ^
| |           |__ the channel string, 9 bytes
| |
| |__ the length of the channel string (it's a u32, so 4 bytes)
|
|__ the "j" tag
```

```
Message::Leave { channel: "mychannel".to_string() }

becomes

l 09 00 00 00 m y c h a n n e l
^ ^           ^
| |           |__ the channel string, 9 bytes
| |
| |__ the length of the channel string (it's a u32, so 4 bytes)
|
|__ the "l" tag (this is the only difference compared to Message::Join)
```

```
Message::Say { channel: "mychannel".to_string(), message: "hello".to_string() }

becomes

S a y 09 00 00 00 m y c h a n n e l 05 00 00 00 h e l l o
^     ^           ^                 ^           ^
|     |           |                 |           |
|     |           |                 |           |__ the message field
|     |           |                 |
|     |           |                 |__ the length of the message field
|     |           |
|     |           |__ the channel string, 9 bytes
|     |
|     |__ the length of the channel string (it's a u32, so 4 bytes)
|
|__ because no tag was specified, it will use the name of the variant.
```

Here's a simple struct:

```rust
#[derive(ToBytes, FromBytes)]
struct MyStruct {
    number: u16,
    bytes: [u8; 8],
    link: String,
    range: RangeInclusive<u8>
}
```

```
MyStruct {
    number: 12,
    bytes: vec![7, 8, 9],
    text: "hello".to_string(),
    range: 5..=22,
};

becomes

0c 00 03 00 00 00 07 08 09 05 00 00 00 h e l l o 05 00 00 00 16 00 00 00
^     ^           ^        ^           ^         ^           ^
|     |           |        |           |         |           |
|     |           |        |           |         |           |__ end of range
|     |           |        |           |         |
|     |           |        |           |         |__ start of range
|     |           |        |           |
|     |           |        |           |__ text
|     |           |        |
|     |           |        |__ length of text
|     |           |
|     |           |__ bytes[0]
|     |
|     |__ length of bytes
|
|__ number
```

By default, structs are untagged. If you wish to tag them, that can be done:

```rust
#[derive(ToBytes, FromBytes)]
#[tag("hello")]
struct MyOtherStruct { value: u32 };
```

```
MyOtherStruct { value: 123 }

becomes

h e l l o 7b 00 00 00
^         ^
|         |__ value
|
|__ tag
```

Without the tag, it would just be

```
7b 00 00 00
```

### Hand rolling serialization

By default, when serializing the String type, it uses a u32 to indicate the length (number of bytes) of the serialized string.

This example shows how to use a u8 to describe the length, saving 3 precious bytes.

```rust
use buffin::{FromBytes, ToBytes};
use eyre::Result;
use nom::{
    IResult,
    bytes::streaming::take,
    error::{Error, ErrorKind},
    number::streaming::le_u8,
};

#[derive(Debug)]
struct ShortString(String);

impl ToBytes for ShortString {
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize> {
        // Buffin instances are dirt cheap to create, so generally to_bytes will create one.
        let mut buffer = Buffin::new(buffer);

        let len = self.0.bytes().len();
        if len > u8::MAX as usize {
            eyre::bail!("the contained string is too long");
        }

        // First we add the length of the string.
        buffer.add(&(len as u8))?;

        // Then we add the string itself.
        buffer.add_bytes(self.0.as_bytes())?;

        // In the end, this function needs to return how many bytes were used.
        Ok(buffer.len())
    }
}

impl FromBytes for ShortString {
    fn from_bytes(buffer: &[u8]) -> IResult<&[u8], Self> {
        // FromBytes uses nom to parse.

        // First, fetch the length of the short string.
        let (buffer, len) = le_u8(buffer)?;

        // Then fetch the bytes, based on thel length.
        let (buffer, bytes) = take(len)(buffer)?;

        // Lastly, try to parse the string, and on success,
        // return a (remainder, ShortString) tuple, wrapped in Ok.
        match String::from_utf8(bytes.to_vec()) {
            Ok(s) => Ok((buffer, ShortString(s))),
            Err(_) => IResult::Err(nom::Err::Failure(Error {
                input: buffer,
                code: ErrorKind::Fail,
            })),
        }
    }
}

// snip //

let s = ShortString("Hello".to_string());

let mut buffer = [0; 1024];
let mut buffer = Buffin::new(&mut buffer);

buffer.add(&s).expect("failed to add short string");

// Show how many bytes we're using. Should be 6.
// A normal String would have used 9.
println!("buffer length: {}", buffer.len());

let parsed_s = buffer.pop::<ShortString>().expect("failed to parse");

println!("Short string: {parsed_s:?}");
```
