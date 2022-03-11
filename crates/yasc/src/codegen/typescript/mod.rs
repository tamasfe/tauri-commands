#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]

use crate::{
    collection::Collection,
    util::{docs_of, type_name_of, SchemaObjectExt, StringWriter},
};
use anyhow::{anyhow, Context};
use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};
use url::Url;

#[derive(Debug, Clone)]
pub struct TypeScriptGeneratorOptions {
    pub export_definitions: bool,
    pub use_interface: bool,
}

impl Default for TypeScriptGeneratorOptions {
    fn default() -> Self {
        Self {
            export_definitions: true,
            use_interface: true,
        }
    }
}

#[derive(Default)]
pub struct TypeScriptGenerator {
    options: TypeScriptGeneratorOptions,
    collection: Collection,
}

impl TypeScriptGenerator {
    #[must_use]
    pub fn new(collection: Collection) -> Self {
        Self::new_with_options(collection, TypeScriptGeneratorOptions::default())
    }

    #[must_use]
    pub fn new_with_options(collection: Collection, options: TypeScriptGeneratorOptions) -> Self {
        Self {
            options,
            collection,
        }
    }

    pub fn generate_definition(
        &self,
        id: &Url,
        type_name: Option<String>,
        out: &mut StringWriter,
    ) -> Result<(), anyhow::Error> {
        let schemas = self.collection.read();

        let schema = schemas
            .get(id)
            .ok_or_else(|| anyhow!(r#"Schema "{}" does not exist!"#, id))?;

        let type_name = type_name
            .or_else(|| type_name_of(schema, Some(id)))
            .ok_or_else(|| anyhow!(r#"Cannot figure out a type name for schema "{}""#, id))?;

        let opts = self.options.clone();

        if let Some(docs) = docs_of(schema).map(str::trim) {
            if !docs.is_empty() {
                out.push_str("/**\n");

                for line in docs.split('\n') {
                    out.push_str(" * ");
                    out.push_str(line);
                    out.push_str("\n");
                }

                out.push_str(" */\n");
            }
        }

        if opts.export_definitions {
            out.push_str("export ");
        }

        if schema.is_single_object() && opts.use_interface {
            out.push_str("interface ");
            out.push_str(&type_name);
            out.push_str(" ");
        } else {
            out.push_str("type ");
            out.push_str(&type_name);
            out.push_str(" = ");
        }

        if schema.is_ref() {
            self.generate_name_or_type(schema, out)
                .with_context(|| format!(r#"Failed to generate type for schema {}"#, id))?;
        } else {
            self.generate_type(schema, out)
                .with_context(|| format!(r#"Failed to generate type for schema {}"#, id))?;
        }

        Ok(())
    }

    pub fn generate_type(
        &self,
        schema: &SchemaObject,
        out: &mut StringWriter,
    ) -> Result<(), anyhow::Error> {
        if let Some(c) = &schema.const_value {
            serde_json::to_writer_pretty(&mut *out, c)?;
            return Ok(());
        }

        if let Some(enum_values) = &schema.enum_values {
            let mut first = true;

            for value in enum_values {
                if !first {
                    out.push_str(" | ");
                }
                first = false;
                serde_json::to_writer_pretty(&mut *out, value)?;
            }
            return Ok(());
        }

        let mut subschemas_written = false;

        if let Some(subschemas) = &schema.subschemas {
            let mut optional_out = StringWriter::new();

            if let Some(oneof_schemas) = &subschemas.one_of {
                for oneof_schema in oneof_schemas {
                    if !optional_out.is_empty() {
                        optional_out.push_str(" | ");
                    }

                    match oneof_schema {
                        Schema::Bool(b) => {
                            if *b {
                                optional_out.push_str("unknown");
                            } else {
                                optional_out.push_str("never");
                            }
                        }
                        Schema::Object(oneof_obj) => {
                            self.generate_name_or_type(oneof_obj, &mut optional_out)?;
                        }
                    }
                    subschemas_written = true;
                }
            }

            if let Some(anyof_schemas) = &subschemas.any_of {
                for anyof_schema in anyof_schemas {
                    if !optional_out.is_empty() {
                        optional_out.push_str(" | ");
                    }

                    match anyof_schema {
                        Schema::Bool(b) => {
                            if *b {
                                optional_out.push_str("unknown");
                            } else {
                                optional_out.push_str("never");
                            }
                        }
                        Schema::Object(anyof_obj) => {
                            self.generate_name_or_type(anyof_obj, &mut optional_out)?;
                        }
                    }

                    subschemas_written = true;
                }
            }

            if !optional_out.is_empty() {
                out.push_str("(");
                out.push_str(&optional_out.take());
                out.push_str(")");
            }

            let mut required_out = StringWriter::new();

            if let Some(allof_schemas) = &subschemas.all_of {
                if !optional_out.is_empty() || optional_out.bytes_written() > 0 {
                    optional_out.push_str(" & ");
                }

                for allow_schema in allof_schemas {
                    if !optional_out.is_empty() {
                        optional_out.push_str(" | ");
                    }

                    match allow_schema {
                        Schema::Bool(b) => {
                            if *b {
                                optional_out.push_str("unknown");
                            } else {
                                optional_out.push_str("never");
                            }
                        }
                        Schema::Object(allow_obj) => {
                            self.generate_name_or_type(allow_obj, &mut required_out)?;
                        }
                    }

                    subschemas_written = true;
                }

                if !required_out.is_empty() {
                    out.push_str(&required_out.finish());
                }
            }
        }

        match &schema.instance_type {
            Some(s) => {
                if subschemas_written {
                    out.push_str(" & ");
                }

                match s {
                    SingleOrVec::Single(s) => {
                        self.generate_instance_type(schema, **s, out)?;
                    }
                    SingleOrVec::Vec(types) => {
                        let mut first = true;

                        for ty in types {
                            if !first {
                                out.push_str(" | ");
                            }
                            first = false;

                            self.generate_instance_type(schema, *ty, out)?;
                        }
                    }
                }
                Ok(())
            }
            _ => {
                if subschemas_written {
                    Ok(())
                } else {
                    self.generate_instance_type(schema, InstanceType::Object, out)?;
                    Ok(())
                }
            }
        }
    }

    fn generate_instance_type(
        &self,
        schema: &SchemaObject,
        ty: InstanceType,
        out: &mut StringWriter,
    ) -> Result<(), anyhow::Error> {
        match ty {
            InstanceType::Null => out.push_str("null"),
            InstanceType::Boolean => out.push_str("boolean"),
            InstanceType::Number | InstanceType::Integer => out.push_str("number"),
            InstanceType::String => out.push_str("string"),
            InstanceType::Object => {
                out.push_str("{\n");

                match &schema.object {
                    Some(obj) => {
                        let mut additional_types = StringWriter::new();

                        if let Some(additional_props) = &obj.additional_properties {
                            match &**additional_props {
                                Schema::Bool(true) => {
                                    additional_types.push_str("unknown");
                                }
                                Schema::Object(additional_obj) => {
                                    self.generate_name_or_type(
                                        additional_obj,
                                        &mut additional_types,
                                    )?;
                                }
                                Schema::Bool(_) => {}
                            }
                        }

                        for obj in obj.pattern_properties.values() {
                            match obj {
                                Schema::Bool(true) => {
                                    if !additional_types.is_empty() {
                                        additional_types.push_str(" | ");
                                    }

                                    additional_types.push_str("unknown");
                                }
                                Schema::Object(pattern_obj) => {
                                    if !additional_types.is_empty() {
                                        additional_types.push_str(" | ");
                                    }

                                    self.generate_name_or_type(pattern_obj, &mut additional_types)?;
                                }
                                Schema::Bool(_) => {}
                            }
                        }

                        if !additional_types.is_empty() {
                            out.push_str("[key: string]: ");
                            out.push_str(&additional_types.finish());
                            out.push_str(";\n");
                        }

                        for (prop_name, prop_schema) in &obj.properties {
                            let required = obj.required.contains(prop_name);

                            if let Schema::Object(o) = &prop_schema {
                                if let Some(docs) = docs_of(o).map(str::trim) {
                                    if !docs.is_empty() {
                                        out.push_str("/**\n");

                                        for line in docs.split('\n') {
                                            out.push_str(" * ");
                                            out.push_str(line);
                                            out.push_str("\n");
                                        }

                                        out.push_str(" */\n");
                                    }
                                }
                            }

                            out.push_str(prop_name);
                            if !required {
                                out.push_str("?");
                            }

                            out.push_str(": ");

                            match prop_schema {
                                Schema::Bool(b) => {
                                    if *b {
                                        out.push_str("unknown;");
                                    } else {
                                        out.push_str("never;");
                                    }
                                }
                                Schema::Object(prop_object) => {
                                    self.generate_name_or_type(prop_object, out)?;
                                    out.push_str(";\n");
                                }
                            }
                        }
                    }
                    None => {
                        out.push_str("[key: string]: unknown;");
                    }
                }

                out.push_str("}");
            }
            InstanceType::Array => match &schema.array {
                Some(arr) => match &arr.items {
                    Some(arr) => match arr {
                        SingleOrVec::Single(v) => match &**v {
                            Schema::Bool(b) => {
                                if *b {
                                    out.push_str("Array<unknown>");
                                } else {
                                    out.push_str("Array<never>");
                                }
                            }
                            Schema::Object(array_items_schema) => {
                                out.push_str("Array<");
                                self.generate_name_or_type(array_items_schema, out)?;
                                out.push_str(">");
                            }
                        },
                        SingleOrVec::Vec(item_schemas) => {
                            out.push_str("[");

                            for item_schema in item_schemas {
                                match item_schema {
                                    Schema::Bool(b) => {
                                        if *b {
                                            out.push_str("unknown");
                                        } else {
                                            out.push_str("never");
                                        }
                                    }
                                    Schema::Object(array_items_schema) => {
                                        self.generate_name_or_type(array_items_schema, out)?;
                                    }
                                }
                                out.push_str(",");
                            }

                            out.push_str("]");
                        }
                    },
                    None => out.push_str("Array<unknown>"),
                },
                None => out.push_str("Array<unknown>"),
            },
        };

        Ok(())
    }

    pub fn generate_name_or_type(
        &self,
        schema: &SchemaObject,
        out: &mut StringWriter,
    ) -> Result<(), anyhow::Error> {
        if let Some(Ok(r)) = schema
            .reference
            .as_ref()
            .map(|r| r.replace("#/definitions/", "root://").parse())
        {
            let schemas = self.collection.read();

            if let Some(name) = type_name_of(
                schemas
                    .get(&r)
                    .ok_or_else(|| anyhow!(r#"Schema not found, but it should exist: "{}""#, &r))?,
                Some(&r),
            ) {
                out.push_str(&name);
                return Ok(());
            }
        } else if schema.is_ref() {
            return Err(anyhow!(
                "schema reference is not absolute, but it should be."
            ));
        }
        self.generate_type(schema, out)
    }
}

impl TypeScriptGenerator {
    #[must_use]
    pub fn collection(&self) -> Collection {
        self.collection.clone()
    }
}
