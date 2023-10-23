use std::convert::Infallible;

use anyhow::Error;
use async_trait::async_trait;
use grafana_plugin_sdk::backend::{
    CheckHealthRequest, CheckHealthResponse, CollectMetricsRequest, CollectMetricsResponse,
    DiagnosticsService, MetricsPayload,
};
use prometheus::{Encoder, TextEncoder};
use thiserror::Error;
use tracing::error;

use crate::service::Service;

#[derive(Debug, Error)]
#[error("Error collecting metrics of OpenARK: {}", .reason)]
pub struct CollectMetricsError {
    reason: Error,
}

impl From<::prometheus::Error> for CollectMetricsError {
    fn from(reason: ::prometheus::Error) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

#[async_trait]
impl DiagnosticsService for Service {
    type CheckHealthError = Infallible;
    type CollectMetricsError = CollectMetricsError;

    async fn check_health(
        &self,
        CheckHealthRequest { plugin_context, .. }: CheckHealthRequest,
    ) -> Result<CheckHealthResponse, <Self as DiagnosticsService>::CheckHealthError> {
        match self.namespaced_with_ctx(&plugin_context).await {
            Ok(service) => Ok(CheckHealthResponse::ok(format!(
                "OpenARK with namespace {namespace:?} is ready!",
                namespace = &service.namespace,
            ))),
            Err(reason) => Ok(CheckHealthResponse::error(reason.to_string())),
        }
    }

    async fn collect_metrics(
        &self,
        _request: CollectMetricsRequest,
    ) -> Result<CollectMetricsResponse, Self::CollectMetricsError> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        encoder.encode(&self.metrics.gather(), &mut buffer)?;

        Ok(CollectMetricsResponse::new(Some(
            MetricsPayload::prometheus(buffer),
        )))
    }
}
