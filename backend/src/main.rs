mod data;
mod service;

#[::grafana_plugin_sdk::main(
    services(data),
    init_subscriber = true,
    shutdown_handler = "0.0.0.0:10002"
)]
async fn plugin() -> Service {
    self::service::Service::default()
}
