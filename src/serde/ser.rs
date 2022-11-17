use serde::{
    ser::{
        self, Impossible, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
        SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize,
};

use super::error::{Error, Result};

fn emit_new_line(output: &mut String) {
    output.push('\n');
}

fn emit_header(output: &mut String, header: &str) {
    output.push('[');
    output.push_str(header);
    output.push(']');
}

fn emit_key(output: &mut String, key: &str) {
    output.push_str(key);
    output.push('=');
}

/// Will serialize a map of header and entry sequence
pub struct HeaderMapSerializer<'a> {
    // This string starts empty and JSON is appended as values are serialized.
    output: &'a mut String,
    new_line: bool,
}

impl<'a> HeaderMapSerializer<'a> {
    pub fn new(output: &'a mut String) -> Self {
        Self {
            output,
            new_line: false,
        }
    }
}

impl<'a, 'b> ser::Serializer for &'b mut HeaderMapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        return Err(Error::ExpectedMap);
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        return Ok(());
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        emit_header(self.output, name);

        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    /// We serialize new-type struct as single header. The type must be a section content.
    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        emit_header(self.output, name);

        value.serialize(&mut SectionSerializer::new(self.output, false))
    }

    /// Serialize the variant as a single header. The new-type must be a section content.
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        emit_header(self.output, variant);

        value.serialize(&mut ValueSerializer::new(self.output))
    }

    /// Serialize as a sequence of sections.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    /// Serialize as a sequence of sections.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    /// Serialize as a sequence of sections. The name is ignored.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    /// Serialize as a sequence of sections. The name is ignored.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(self)
    }

    /// Serialize as a map of section header and section content.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    /// Serialize as a map of section header and section content. The name is ignored.
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    /// Serialize as a map of section header and section content. The name and variant are ignored.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(self)
    }
}

impl<'a, 'b> SerializeSeq for &'b mut HeaderMapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut SectionSerializer::new(self.output, self.new_line))?;

        if !self.new_line {
            self.new_line = true;
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, 'b> SerializeTuple for &'b mut HeaderMapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut SectionSerializer::new(self.output, self.new_line))?;

        if !self.new_line {
            self.new_line = true;
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, 'b> SerializeTupleStruct for &'b mut HeaderMapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(&mut SectionSerializer::new(self.output, self.new_line))?;

        if !self.new_line {
            self.new_line = true;
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<'a, 'b> SerializeTupleVariant for &'b mut HeaderMapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(&mut SectionSerializer::new(self.output, self.new_line))?;

        if !self.new_line {
            self.new_line = true;
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<'a, 'b> SerializeMap for &'b mut HeaderMapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        key.serialize(&mut HeaderSerializer::new(self.output, self.new_line))?;

        self.new_line = true;

        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(&mut ValueSerializer::new(self.output))
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, 'b> SerializeStruct for &'b mut HeaderMapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        emit_header(self.output, key);
        value.serialize(&mut ValueSerializer::new(self.output))
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, 'b> SerializeStructVariant for &'b mut HeaderMapSerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        emit_key(self.output, key);
        value.serialize(&mut ValueSerializer::new(self.output))
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

/// The start of the serializer, serialize a new header
pub struct SectionSerializer<'a> {
    // This string starts empty and JSON is appended as values are serialized.
    output: &'a mut String,
    new_line: bool,
}

impl<'a> SectionSerializer<'a> {
    pub fn new(output: &'a mut String, new_line: bool) -> Self {
        Self { output, new_line }
    }
}

impl<'a, 'b> ser::Serializer for &'b mut SectionSerializer<'a> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;

    type SerializeTuple = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    type SerializeMap = Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Err(Error::ExpectedMap)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        emit_header(self.output, name);

        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        emit_header(self.output, name);
        emit_key(self.output, variant);

        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        emit_header(self.output, name);

        value.serialize(&mut EntrySerializer::new(self.output))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        emit_key(self.output, variant);

        value.serialize(&mut EntrySerializer::new(self.output))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::ExpectedMap)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

pub struct HeaderSerializer<'a> {
    output: &'a mut String,
    new_line: bool,
}

impl<'a> HeaderSerializer<'a> {
    pub fn new(output: &'a mut String, new_line: bool) -> Self {
        Self { output, new_line }
    }
}

impl<'a, 'b> ser::Serializer for &'b mut HeaderSerializer<'a> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;

    type SerializeTuple = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    type SerializeMap = Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

/// Serializes one or more entries
pub struct EntrySerializer<'a> {
    output: &'a mut String,
}

impl<'a> EntrySerializer<'a> {
    pub fn new(output: &mut String) -> Self {
        Self { output }
    }
}

impl<'a, 'b> ser::Serializer for &'b mut EntrySerializer<'a> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

impl<'a, 'b> SerializeSeq for &'b mut EntrySerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<'a, 'b> SerializeTuple for &'b mut EntrySerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}
impl<'a, 'b> SerializeTupleStruct for &'b mut EntrySerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

impl<'a, 'b> SerializeTupleVariant for &'b mut EntrySerializer<'a> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok> {
        todo!()
    }
}

pub struct ValueSerializer<'a> {
    output: &'a mut String,
}

impl<'a> ValueSerializer<'a> {
    pub fn new(output: &'a mut String) -> Self {
        Self { output }
    }
}

impl<'a, 'b> ser::Serializer for &'b mut ValueSerializer<'a> {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;

    type SerializeTuple = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    type SerializeMap = Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = Impossible<Self::Ok, Self::Error>;

    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

pub fn to_string<T: Serialize + Sized>(value: &T) -> Result<&str> {
    todo!()
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
    fn test_struct_nested() {
        #[derive(Serialize)]
        struct Wrap {
            inner: Test,
            other: Test,
        }

        #[derive(Serialize)]
        struct Test {
            #[serde(flatten)]
            int: u32,
            #[serde(flatten)]
            seq: Vec<&'static str>,
        }

        let test = Wrap {
            inner: Test {
                int: 1,
                seq: vec!["a", "b"],
            },
            other: Test {
                int: 1,
                seq: vec!["a", "b"],
            },
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
        assert_eq!(to_string(&u), Err(Error::UnsupportedType));

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
