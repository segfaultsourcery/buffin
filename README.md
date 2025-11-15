# Buffin

This crate allows you to do quick and easy (de)serialization of data, originally meant for passing data to and from small embedded applications.

## Examples

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
