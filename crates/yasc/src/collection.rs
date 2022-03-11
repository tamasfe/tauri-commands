use crate::{HashMap, HashSet};
use anyhow::Context as ErrorContext;
use async_recursion::async_recursion;
use parking_lot::{RwLock, RwLockReadGuard};
use schemars::{schema::SchemaObject, visit::Visitor};
use std::sync::Arc;
use url::Url;

#[derive(Debug, Default, Clone)]
pub struct Collection {
    schemas: Arc<RwLock<HashMap<Url, SchemaObject>>>,
}

impl Collection {
    pub fn len(&self) -> usize {
        self.schemas.read_recursive().len()
    }

    pub fn is_empty(&self) -> bool {
        self.schemas.read_recursive().is_empty()
    }

    pub fn read(&self) -> RwLockReadGuard<'_, HashMap<Url, SchemaObject>> {
        self.schemas.read_recursive()
    }
}

impl Collection {
    pub fn add_from_generator(&self, generator: &schemars::gen::SchemaGenerator) {
        let mut schemas = self.schemas.write();

        for (name, schema) in generator.definitions() {
            let mut cr = CollectReferences::default();

            let mut schema = schema.clone().into_object();
            cr.visit_schema_object(&mut schema);

            let mut replacer = ReplaceReferences::default();

            if schema.metadata().title.is_none() {
                schema.metadata().title = Some(name.clone());
            }

            for r in cr.references {
                let url = r.replace("#/definitions/", "root://");
                replacer.references.insert(r, url.parse().unwrap());
            }

            schemas.insert(format!("root://{name}").parse().unwrap(), schema);
        }
    }

    #[async_recursion]
    pub async fn add_schema_with_id(
        &self,
        root: &Url,
        mut schema: SchemaObject,
    ) -> Result<(), anyhow::Error> {
        if self.schemas.read().contains_key(root) {
            tracing::trace!(id = %root, "schema already exists");
            return Ok(());
        }

        if let Some(s) = schema.metadata.as_ref().and_then(|m| m.id.as_ref()) {
            if let Ok(id) = Url::parse(s) {
                if self.schemas.read().contains_key(&id) {
                    tracing::trace!(%id, "schema already exists");
                    return Ok(());
                }
            }
        }

        let mut collector = CollectReferences::default();
        collector.visit_schema_object(&mut schema);

        let mut replacer = ReplaceReferences::default();

        for r in collector.references {
            let mut root_url = None;

            if let Ok(u) = Url::parse(&r) {
                root_url = Some(u);
            }

            if let (None, Some(id)) = (
                &root_url,
                schema.metadata.as_ref().and_then(|m| m.id.as_ref()),
            ) {
                if let Ok(u) = Url::parse(id).and_then(|u| u.join(&r)) {
                    root_url = Some(u);
                }
            }

            let target_url = match root_url {
                Some(r) => r,
                None => root.join(&r)?,
            };

            replacer.references.insert(r, target_url.clone());

            if self.schemas.read().contains_key(&target_url) {
                tracing::trace!(id = %target_url, "schema already exists");
                continue;
            }

            self.add_schema_with_id(
                &target_url,
                retrieve_schema(&target_url).await.with_context(|| {
                    format!(
                        r#"Failed to resolve reference "{}" in schema "{}""#,
                        &target_url, &root
                    )
                })?,
            )
            .await?;
        }

        replacer.visit_schema_object(&mut schema);

        self.schemas
            .write()
            .insert(id_for_schema(root, &schema), schema.clone());

        tracing::info!(id = %root, "added schema");

        Ok(())
    }
}

#[tracing::instrument(skip(url), fields(%url), level = "debug")]
async fn retrieve_schema(url: &Url) -> Result<SchemaObject, anyhow::Error> {
    unimplemented!()
}

fn id_for_schema(url: &Url, schema: &SchemaObject) -> Url {
    if let Some(Ok(u)) = schema
        .metadata
        .as_ref()
        .and_then(|m| m.id.as_deref().map(Url::parse))
    {
        return u;
    }

    url.clone()
}

#[derive(Default)]
struct CollectReferences {
    references: HashSet<String>,
}

impl schemars::visit::Visitor for CollectReferences {
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        if let Some(s) = &schema.reference {
            if !self.references.contains(s) {
                self.references.insert(s.clone());
            }
        } else {
            schemars::visit::visit_schema_object(self, schema)
        }
    }
}

#[derive(Default)]
struct ReplaceReferences {
    references: HashMap<String, Url>,
}

impl schemars::visit::Visitor for ReplaceReferences {
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        if let Some(s) = &mut schema.reference {
            if let Some(r) = self.references.get(s) {
                tracing::trace!(new = %r, old = %s, "replaced reference in schema");
                *s = r.to_string();
            }
        } else {
            schemars::visit::visit_schema_object(self, schema)
        }
    }
}
