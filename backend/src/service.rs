use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use dash_query_provider::{QueryClient, QueryClientArgs};
use grafana_plugin_sdk::{
    backend::PluginContext,
    data::{ArrayRefIntoField, Frame},
    prelude::IntoFrame,
};
use prometheus::Registry;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct Service {
    pub metrics: Registry,
    namespaces: Arc<RwLock<HashMap<String, NamespacedService>>>,
}

impl Service {
    async fn namespaced(&self, namespace: &str) -> Result<NamespacedService> {
        let namespaces = self.namespaces.read().await;
        match namespaces.get(namespace) {
            Some(service) => Ok(service.clone()),
            None => {
                drop(namespaces);

                let namespace: String = namespace.into();
                let service = NamespacedService::try_new(namespace.clone()).await?;
                {
                    let mut namespaces = self.namespaces.write().await;
                    namespaces.insert(namespace, service.clone());
                }
                Ok(service)
            }
        }
    }

    pub async fn namespaced_with_ctx(
        &self,
        plugin_context: &PluginContext,
    ) -> Result<NamespacedService> {
        self.namespaced(plugin_context.parse_namespace()?).await
    }
}

#[derive(Clone)]
pub struct NamespacedService {
    client: Arc<QueryClient>,
    pub namespace: String,
}

impl NamespacedService {
    async fn try_new(namespace: String) -> Result<Self> {
        Ok(Self {
            client: Arc::new(
                QueryClient::try_new(&QueryClientArgs {
                    namespace: Some(namespace.clone()),
                })
                .await?,
            ),
            namespace,
        })
    }
}

impl NamespacedService {
    pub async fn sql(&self, name: &str, sql: &str) -> Result<Frame> {
        let records = self.client.sql(sql).await?.collect().await?;

        let frame = records
            .into_iter()
            .flat_map(|record| {
                record
                    .schema()
                    .all_fields()
                    .iter()
                    // TODO: struct field support (flatten)
                    .filter(|field| field.name() == "timestamp")
                    // TODO: field aggregation
                    .next()
                    .into_iter()
                    .filter_map(|field| {
                        record.column_by_name(field.name()).and_then(|array| {
                            array
                                .as_ref()
                                // TODO: dynamic slice
                                // .slice(0, 10)
                                .try_into_field(field.name())
                                .ok()
                        })
                    })
                    .collect::<Vec<_>>()
            })
            // TODO: field aggregation
            .skip(1)
            .next()
            .into_iter()
            .collect::<Vec<_>>()
            .into_frame(name);

        Ok(frame)
    }
}

pub trait ParseNamespace {
    fn parse_namespace(&self) -> Result<&str>;
}

impl ParseNamespace for PluginContext {
    fn parse_namespace(&self) -> Result<&str> {
        self.datasource_instance_settings
            .as_ref()
            .and_then(|ds| ds.json_data.get("namespace"))
            .and_then(|value| value.as_str())
            .ok_or_else(|| anyhow!("Empty Namespace"))
    }
}
