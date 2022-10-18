use serde::{ser, Serialize};

use super::error::{Error, Result};

/// State of the serialization.
enum State {
    /// Start of file
    NoHeader,
    // Start of section
    NewSection,
    /// Under an header
    Header,
    /// A key was inserted
    Key,
}

pub struct Serializer {
    // This string starts empty and JSON is appended as values are serialized.
    output: String,
    state: State,
}

impl Serializer {
    fn write_new_line(&mut self) {
        self.output.push('\n');
    }

    fn write_header(&mut self, header: &str) {
        self.output.push('[');
        self.output.push_str(header);
        self.output.push(']');
    }

    fn write_key(&mut self, key: &str) {
        self.output.push_str(key);
        self.output.push('=');
    }
}

// By convention, the public API of a Serde serializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
//
/// This basic serializer supports only `to_string`.
///
/// # Errors
///
/// TODO
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
        state: State::NoHeader,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output += &v.to_string();
        Ok(())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<()> {
        self.output += v;
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    /// Serialize an `unit_struct` `struct Unit` or `PhantomData<T>`:
    fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
        match self.state {
            State::NoHeader => {
                self.write_header(name);
                self.state = State::NewSection;
            }
            State::NewSection => {
                self.write_new_line();
                self.write_header(name);
            }
            State::Header => {
                self.write_new_line();
                self.write_key(name);
            }
            State::Key => {
                self.output.push_str(name);

                self.state = State::Header;
            }
        }

        Ok(())
    }

    /// Serializes `E::A` and `E::B` in `enum E { A, B }`:
    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        match self.state {
            State::NoHeader => {
                self.write_header(name);
                self.write_new_line();
                self.write_key(variant);

                self.state = State::NewSection;
            }
            State::NewSection => {
                self.write_new_line();
                self.write_header(variant);
                self.write_new_line();
                self.write_key(variant);
            }
            State::Header => {
                self.write_new_line();
                self.write_key(variant);
            }
            State::Key => {
                self.output.push_str(name);

                self.state = State::Header;
            }
        }

        Ok(())
    }

    /// Serializes `struct Millimeters(u8)`:
    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.state {
            State::NoHeader => {
                self.write_header(name);
                self.write_new_line();

                self.state = State::Header;

                value.serialize(&mut *self)?;
                self.output.push('=');

                self.state = State::NewSection;
            }
            State::NewSection => {
                self.write_new_line();
                self.write_header(name);
                self.write_new_line();

                self.state = State::Header;

                value.serialize(&mut *self)?;
                self.output.push('=');

                self.state = State::NewSection;
            }
            State::Header => {
                self.write_new_line();
                self.write_key(name);
                self.state = State::Key;

                value.serialize(&mut *self)?;

                self.state = State::Header;
            }
            State::Key => {
                value.serialize(&mut *self)?;

                self.state = State::Header;
            }
        }

        Ok(())
    }

    /// Serializes `E::N` in `enum E { N(u8) }`:
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.state {
            State::NoHeader => {
                self.write_header(name);
                self.write_new_line();

                self.write_key(variant);

                self.state = State::Key;
                value.serialize(&mut *self)?;

                self.state = State::NewSection;
            }
            State::NewSection => {
                self.write_new_line();
                self.write_header(name);
                self.write_new_line();

                self.write_key(variant);

                self.state = State::Key;
                value.serialize(&mut *self)?;

                self.state = State::NewSection;
            }
            State::Header => {
                self.write_new_line();
                self.write_key(name);
                self.state = State::Key;

                value.serialize(&mut *self)?;

                self.state = State::Header;
            }
            State::Key => {
                value.serialize(&mut *self)?;

                self.state = State::Header;
            }
        }

        Ok(())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    /// Serialize `E::T` in `enum E { T(u8, u8) }`.
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        match self.state {
            State::NoHeader => {
                self.write_header(name);
                self.write_new_line();

                self.write_key(variant);

                self.state = State::NewSection;
            }
            State::NewSection => {
                self.write_new_line();
                self.write_header(name);
                self.write_new_line();

                self.write_key(variant);

                self.state = State::NewSection;
            }
            State::Header => {
                self.write_new_line();
                self.write_key(variant);

                self.state = State::Key;
            }
            State::Key => {}
        }

        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        match self.state {
            State::NoHeader => {
                self.write_header(name);

                self.state = State::Header;
            }
            State::NewSection => {
                self.write_new_line();
                self.write_header(name);

                self.state = State::Header;
            }
            State::Header => {}
            State::Key => {
                return Err(Error::NestingNotSupported);
            }
        }

        Ok(self)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    //
    // NOTE: the Struct serialize also checks for the Key state which can be simplified
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        match self.state {
            State::NoHeader => {
                self.write_header(name);

                self.state = State::Header;
            }
            State::NewSection => {
                self.write_new_line();
                self.write_header(name);

                self.state = State::Header;
            }
            State::Header => {}
            State::Key => {
                return Err(Error::NestingNotSupported);
            }
        }

        Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
//
// TODO: serialize with an index for the sequence
impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.is_empty() && !self.output.ends_with('=') {
            self.output.push(';');
        }

        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.is_empty() && !self.output.ends_with('=') {
            self.output.push(';');
        }

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        if self.output.ends_with(';') {
            self.output.pop();
        }

        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.is_empty() && !self.output.ends_with('=') {
            self.output.push(';');
        }

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.is_empty() && !self.output.ends_with('=') {
            self.output.push(';');
        }

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_new_line();

        key.serialize(&mut **self)?;
        self.output.push('=');

        self.state = State::Key;

        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.output.push('\n');

        self.state = State::Header;

        Ok(())
    }

    fn end(self) -> Result<()> {
        self.state = State::NewSection;
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.state {
            State::NoHeader => {
                self.write_header(key);

                self.state = State::Header;

                value.serialize(&mut **self)?;

                self.state = State::NewSection;
            }
            State::NewSection => {
                self.write_new_line();

                self.write_header(key);

                self.state = State::Header;

                value.serialize(&mut **self)?;

                self.state = State::NewSection;
            }
            State::Header => {
                self.write_new_line();

                self.write_key(key);

                self.state = State::Key;

                value.serialize(&mut **self)?;

                self.state = State::Header;
            }
            // Cant serialize, since the key will be lost
            State::Key => {
                return Err(Error::NestingNotSupported);
            }
        }

        Ok(())
    }

    fn end(self) -> Result<()> {
        self.state = State::NewSection;

        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.state {
            State::NoHeader => {
                self.write_header(key);

                self.state = State::Header;

                value.serialize(&mut **self)?;

                self.state = State::NewSection;
            }
            State::NewSection => {
                self.write_new_line();

                self.write_header(key);

                self.state = State::Header;

                value.serialize(&mut **self)?;

                self.state = State::NewSection;
            }
            State::Header => {
                self.write_new_line();

                self.write_key(key);

                self.state = State::Key;

                value.serialize(&mut **self)?;

                self.state = State::Header;
            }
            // Cant serialize, since the key will be lost
            State::Key => {
                return Err(Error::NestingNotSupported);
            }
        }

        Ok(())
    }

    fn end(self) -> Result<()> {
        self.state = State::NewSection;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"],
        };

        let expected = "[Test]\nint=1\nseq=a;b";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let u = E::Unit;
        let expected = "[E]\nUnit=";
        assert_eq!(to_string(&u).unwrap(), expected);

        let n = E::Newtype(1);
        let expected = "[E]\nNewtype=1";
        assert_eq!(to_string(&n).unwrap(), expected);

        let t = E::Tuple(1, 2);
        let expected = "[E]\nTuple=1;2";
        assert_eq!(to_string(&t).unwrap(), expected);

        let s = E::Struct { a: 1 };
        let expected = "[E]\na=1";
        assert_eq!(to_string(&s).unwrap(), expected);
    }
}
