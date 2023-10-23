use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use dash_query_provider::{QueryClient, QueryClientArgs};
use grafana_plugin_sdk::{
    data::{ArrayRefIntoField, Frame},
    prelude::IntoFrame,
};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct Service {
    namespaces: Arc<RwLock<HashMap<String, NamespacedService>>>,
}

impl Service {
    pub async fn namespaced(&self, namespace: &str) -> Result<NamespacedService> {
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
}

#[derive(Clone)]
pub struct NamespacedService(Arc<QueryClient>);

impl NamespacedService {
    async fn try_new(namespace: String) -> Result<Self> {
        Ok(Self(Arc::new(
            QueryClient::try_new(&QueryClientArgs {
                namespace: Some(namespace),
            })
            .await?,
        )))
    }
}

impl NamespacedService {
    pub async fn sql(&self, name: &str, sql: &str) -> Result<Frame> {
        let records = self.0.sql(sql).await?.collect().await?;

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
