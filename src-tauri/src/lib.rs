pub(crate) mod commands;
mod cerebro;
pub mod lazy_service;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

use tauri::Manager;
use crate::commands::lazy_process;
use crate::commands::searxng;
use crate::commands::spotify;
use lazy_service::LazyServiceRegistry;

pub fn run() {
    let _guard = commands::errors::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let mut registry = lazy_process::init_registry();

            let config = match commands::config::load_config_sync() {
                Ok(c) => c,
                Err(e) => {
                    warn!("failed to load config, using defaults: {}", e);
                    Default::default()
                }
            };
            if config.boot.lazy_cerebro {
                let binary = if config.cerebro.binary_path.is_empty() {
                    None
                } else {
                    Some(config.cerebro.binary_path.clone())
                };
                let hc_url = format!("{}/health", config.cerebro.base_url);
                lazy_process::register_process(
                    &mut registry,
                    lazy_process::LazyProcessConfig {
                        name: "cerebro".into(),
                        command: binary,
                        args: vec![],
                        idle_timeout_secs: config.cerebro.idle_timeout_secs,
                        healthcheck_url: Some(hc_url),
                        healthcheck_interval_secs: Some(5),
                    },
                );
            }

            let mut watcher = lazy_process::IdleWatcher::new();
            for handle in registry.handles.values() {
                watcher.register(handle.clone());
            }

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                watcher.start_watching(app_handle).await;
            });

            let registry = Arc::new(Mutex::new(registry));
            app.manage(registry);

            // Manage LazyServiceRegistry via tauri::State
            // TODO: Wire LazyServiceRegistry into tauri command handlers
            // (e.g. list_services, start_service) so it's actually used.
            let service_registry = Arc::new(Mutex::new(LazyServiceRegistry::new()));
            app.manage(service_registry);

            // Init SearXNG manager
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = searxng::init_searxng(app_handle).await {
                    warn!(?e, "SearXNG init skipped");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::boot::run_boot_sequence,
            commands::boot::run_boot_step,
            commands::boot::check_cerebro,
            commands::boot::start_cerebro,
            commands::cerebro::query_cerebro,
            commands::cerebro::ask_llm,
            commands::cerebro::check_cerebro_health,
            commands::cerebro::send_to_cerebro,
            commands::cerebro::send_to_cerebro_with_fallback,
            commands::config::load_config,
            commands::config::save_config,
            commands::config::get_config_value,
            lazy_process::lazy_start,
            lazy_process::lazy_stop,
            lazy_process::lazy_is_running,
            lazy_process::lazy_healthcheck,
            lazy_process::lazy_touch,
            lazy_process::lazy_list,
            lazy_process::lazy_get_status,
            searxng::start_searxng,
            searxng::stop_searxng,
            searxng::searxng_status,
            searxng::search_web,
            searxng::init_searxng,
            searxng::search_youtube,
            searxng::play_youtube,
            spotify::is_spotify_available,
            spotify::search_spotify,
            spotify::play_spotify,
            spotify::authorize_spotify_user,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Fatal: failed to run tauri application: {e}");
            std::process::exit(1);
        });
}
