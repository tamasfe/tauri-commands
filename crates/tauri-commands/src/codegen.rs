use std::{borrow::Cow, path::Path};

use heck::ToLowerCamelCase;
use schemars::schema::Schema;
use tauri::Runtime;
use yasc::{codegen::typescript::TypeScriptGenerator, collection::Collection, util::StringWriter};

use crate::Commands;

#[derive(Debug, Default)]
pub struct CommandMeta {
    pub docs: Cow<'static, str>,
    pub args: Vec<CommandArg>,
    pub output_schema: Option<schemars::schema::Schema>,
}

impl CommandMeta {
    fn generate_ts_handler(
        &self,
        cmd_name: &str,
        gen: &TypeScriptGenerator,
        sw: &mut StringWriter,
    ) {
        if !self.docs.is_empty() {
            sw.push_str("/**\n");

            for line in self.docs.split('\n') {
                sw.push_str(" * ");
                sw.push_str(line);
                sw.push_str("\n");
            }

            sw.push_str(" */\n");
        }
        sw.push_str("export function ");
        sw.push_str(&cmd_name.to_lower_camel_case());
        sw.push_str("(");

        let mut msg_obj = StringWriter::default();

        msg_obj.push_str("{");

        for (idx, arg) in self.args.iter().enumerate() {
            if let Schema::Object(s) = &arg.schema {
                sw.push_str(&arg.name);
                sw.push_str(": ");
                gen.generate_name_or_type(s, sw).unwrap();
                sw.push_str(",");
            }

            let idx = idx + 1;
            msg_obj.push_str(&format!("_{idx}: "));
            msg_obj.push_str(&arg.name);
            msg_obj.push_str(",");
        }

        msg_obj.push_str("}");

        sw.push_str("): Promise<");
        if let Some(s) = &self.output_schema {
            match s {
                Schema::Bool(s) => {
                    if *s {
                        sw.push_str("unknown")
                    } else {
                        sw.push_str("never")
                    }
                }
                Schema::Object(s) => gen.generate_name_or_type(s, sw).unwrap(),
            }
        }
        sw.push_str("> {");
        sw.push_str(&format!("return invoke('{cmd_name}', {msg_obj});"));
        sw.push_str("}\n");
    }
}

#[derive(Debug)]
pub struct CommandArg {
    pub name: Cow<'static, str>,
    pub schema: schemars::schema::Schema,
}

impl<R: Runtime> Commands<R> {
    pub fn generate_typescript(&self) -> String {
        let mut sw = StringWriter::default();

        sw.push_str(
            r#"import { invoke } from "@tauri-apps/api";
"#,
        );

        let c = Collection::default();

        c.add_from_generator(&self.schema_gen);

        let gen = TypeScriptGenerator::new(c.clone());

        self.generate_definitions(&c, &gen, &mut sw);

        for (name, cmd) in &self.commands {
            cmd.meta.generate_ts_handler(&*name, &gen, &mut sw);
        }

        sw.finish()
    }

    pub fn write_typescript(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        if let Some(parent) = path.as_ref().parent()  {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path.as_ref(), self.generate_typescript())
    }

    fn generate_definitions(
        &self,
        c: &Collection,
        gen: &TypeScriptGenerator,
        sw: &mut StringWriter,
    ) {
        let schemas = c.read();

        for s in schemas.keys() {
            gen.generate_definition(s, None, sw).unwrap();
            sw.push_str("\n");
        }
    }
}
