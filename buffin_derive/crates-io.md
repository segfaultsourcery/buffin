_This crate should be used together with the [buffin](https://crates.io/crates/buffin) crate._

This crate enables you to automatically derive ToBytes and FromBytes for custom structs and enums, provided all fields within them implement ToBytes and FromBytes, respectively.

```rust
use buffin::Buffin;
use buffin_derive::{FromBytes, ToBytes};

#[derive(ToBytes, FromBytes)]
struct MyStruct {
    number: u16,
    text: String,
}

#[derive(ToBytes, FromBytes)]
enum MyEnum {
    #[tag("vnt1")]
    Variant1 {
        my_struct: MyStruct,
    },
}

// snip //

let my_enum = MyEnum::Variant1 {
    my_struct: MyStruct {
        number: 12,
        text: "letters and things".to_string(),
    }
};

// Grab a bit of memory and stuff it in a Buffin instance.
// This can also be a vec. Keep in mind that the vec will never be resized.
let mut buffer = [0; 1024];
let mut buffer = Buffin::new(&mut buffer);

// Put my_enum in the buffin.
buffer.add(&my_enum).expect("failed to add my_enum");

// Check how many bytes we've used.
println!("buffer length: {}", buffer.len());

// Take it back out again.
let my_parsed_enum = buffer.pop::<MyEnum>().expect("failed to parse");
```

It's possible to serialize multiple things into the same buffer.

```rust
#[derive(Debug, ToBytes, FromBytes)]
enum Message {
    #[tag("j")]
    Join { channel: String },
    #[tag("l")]
    Leave { channel: String },
    #[tag("s")]
    Say { channel: String, message: String }
}

// snip //

let mut buffer = [0; 1024];
let mut buffer = Buffin::new(&mut buffer);

// Put some messages in the buffin.
buffer.add(&Message::Join { channel: "ch1".to_string() }).expect("failed to add message");
buffer.add(&Message::Say { channel: "ch1".to_string(), message: "Woop woop".to_string() }).expect("failed to add message");
buffer.add(&Message::Leave { channel: "ch1".to_string() }).expect("failed to add message");

// Check how many bytes we've used. It should be 37.
println!("buffer length: {}", buffer.len());

// Read them all back out again.
while let Ok(message) = buffer.pop::<Message>() {
    println!("message: {message:?}");
}
```
