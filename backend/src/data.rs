use anyhow::{anyhow, Error};
use async_trait::async_trait;
use futures::stream::FuturesOrdered;
use grafana_plugin_sdk::backend::{
    BoxDataResponseStream, DataQuery, DataQueryError, DataResponse, DataService, QueryDataRequest,
};
use serde::Deserialize;
use thiserror::Error;
use tracing::error;

use crate::service::Service;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    raw_query: Option<String>,
}

#[derive(Debug, Error)]
#[error("Error querying to OpenARK for {}: {}", .ref_id, .reason)]
pub struct QueryError {
    reason: Error,
    ref_id: String,
}

impl DataQueryError for QueryError {
    fn ref_id(self) -> String {
        self.ref_id
    }
}

#[async_trait]
impl DataService for Service {
    type Query = Query;
    type QueryError = QueryError;
    type Stream = BoxDataResponseStream<Self::QueryError>;

    async fn query_data(
        &self,
        QueryDataRequest {
            plugin_context,
            queries,
            ..
        }: QueryDataRequest<<Self as DataService>::Query>,
    ) -> <Self as DataService>::Stream {
        fn panic(
            mut queries: Vec<DataQuery<<Service as DataService>::Query>>,
            reason: Error,
        ) -> <Service as DataService>::Stream {
            Box::pin(
                queries
                    .pop()
                    .map(|DataQuery { ref_id, .. }| async move { Err(QueryError { reason, ref_id }) })
                    .into_iter()
                    .collect::<FuturesOrdered<_>>(),
            )
        }

        let service = match self.namespaced_with_ctx(&plugin_context).await {
            Ok(service) => service,
            Err(reason) => return panic(queries, reason),
        };

        Box::pin(
            queries
                .into_iter()
                .map(
                    |DataQuery {
                         query: Query { raw_query, .. },
                         ref_id,
                         ..
                     }| {
                        let service = service.clone();
                        async move {
                            match match raw_query {
                                Some(raw_query) if !raw_query.is_empty() => {
                                    service.sql("Raw Query", &raw_query).await
                                }
                                Some(_) | None => Err(anyhow!("Empty SQL")),
                            } {
                                Ok(frame) => match frame.check().map_err(Into::into) {
                                    Ok(frame) => Ok(DataResponse::new(ref_id, vec![frame])),
                                    Err(reason) => Err(QueryError { reason, ref_id }),
                                },
                                Err(reason) => Err(QueryError { reason, ref_id }),
                            }
                        }
                    },
                )
                .collect::<FuturesOrdered<_>>(),
        )
    }
}
