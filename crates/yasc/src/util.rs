use std::{io, mem};

use heck::ToUpperCamelCase;
use schemars::schema::{InstanceType, SchemaObject};
use url::Url;

pub trait SchemaObjectExt: private::Sealed {
    fn is_single_object(&self) -> bool;
}

impl SchemaObjectExt for SchemaObject {
    fn is_single_object(&self) -> bool {
        self.reference.is_none()
            && self
                .instance_type
                .as_ref()
                .map(|s| match s {
                    schemars::schema::SingleOrVec::Single(ty) => **ty == InstanceType::Object,
                    schemars::schema::SingleOrVec::Vec(types) => {
                        types.len() == 1 && types.iter().all(|ty| *ty == InstanceType::Object)
                    }
                })
                .unwrap_or(true)
            && self
                .enum_values
                .as_ref()
                .map(|e| e.is_empty())
                .unwrap_or(true)
            && self.const_value.is_none()
            && self.subschemas.is_none()
    }
}

#[derive(Debug, Default)]
pub struct StringWriter {
    written: usize,
    buf: String,
}

impl io::Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written += buf.len();
        self.buf.push_str(
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        );
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl core::fmt::Display for StringWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.buf.fmt(f)
    }
}

impl StringWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_str(&mut self, string: &str) {
        self.written += string.len();
        self.buf.push_str(string);
    }

    pub fn finish(self) -> String {
        self.buf
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn take(&mut self) -> String {
        mem::take(&mut self.buf)
    }

    pub fn bytes_written(&self) -> usize {
        self.written
    }

    pub fn prepend_str(&mut self, string: &str) {
        self.written += string.len();
        self.buf = String::from(string) + &self.buf;
    }
}

pub fn docs_of(schema: &SchemaObject) -> Option<&str> {
    schema
        .metadata
        .as_deref()
        .and_then(|m| m.description.as_deref())
}

pub fn type_name_of(schema: &SchemaObject, id: Option<&Url>) -> Option<String> {
    schema
        .metadata
        .as_ref()
        .and_then(|s| s.title.as_ref().map(|s| s.to_upper_camel_case()))
        .or_else(|| {
            id.and_then(Url::path_segments)
                .and_then(std::iter::Iterator::last)
                .and_then(|segment| segment.split('.').next())
                .map(|s| s.to_upper_camel_case())
        })
}

mod private {
    use schemars::schema::SchemaObject;

    pub trait Sealed {}

    impl Sealed for SchemaObject {}
}
