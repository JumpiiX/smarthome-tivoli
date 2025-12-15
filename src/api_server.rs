use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

use crate::device::{Device, DeviceState};
use crate::state_manager::StateManager;

#[derive(Clone)]
pub struct ApiState {
    pub state_manager: Arc<StateManager>,
}

#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub key: String,
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub page: String,
    pub state: DeviceStateInfo,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DeviceStateInfo {
    OnOff { on: bool },
    Brightness { on: bool, level: u8 },
    WindowCovering { position: u8 },
    Temperature { celsius: f32 },
    FanSpeed { speed: u8 },
}

#[derive(Debug, Deserialize)]
pub struct ToggleRequest {
    pub on: bool,
}

#[derive(Debug, Deserialize)]
pub struct BlindPositionRequest {
    pub position: u8,
}

#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub devices: Vec<DeviceInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl From<&Device> for DeviceInfo {
    fn from(device: &Device) -> Self {
        let device_type = format!("{:?}", device.device_type);
        let state = match &device.state {
            DeviceState::OnOff(on) => DeviceStateInfo::OnOff { on: *on },
            DeviceState::Brightness { on, level } => DeviceStateInfo::Brightness {
                on: *on,
                level: *level,
            },
            DeviceState::WindowCovering { position, .. } => DeviceStateInfo::WindowCovering {
                position: *position,
            },
            DeviceState::Temperature(temp) => DeviceStateInfo::Temperature { celsius: *temp },
            DeviceState::FanSpeed(speed) => DeviceStateInfo::FanSpeed { speed: *speed },
        };

        DeviceInfo {
            key: device.key(),
            id: device.id.clone(),
            name: device.name.clone(),
            device_type,
            page: device.page.clone(),
            state,
        }
    }
}

pub async fn start_api_server(state_manager: Arc<StateManager>, port: u16) -> Result<()> {
    let state = ApiState { state_manager };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/devices", get(list_devices))
        .route("/device/:key", get(get_device))
        .route("/device/:key/state", get(get_device_state))
        .route("/device/:key/toggle", post(toggle_device))
        .route("/device/:key/position", post(set_blind_position))
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    info!("🌐 HTTP API server listening on http://{}", addr);
    info!("   API endpoints:");
    info!("   - GET  /devices                List all devices");
    info!("   - GET  /device/:key            Get device info");
    info!("   - GET  /device/:key/state      Get device state");
    info!("   - POST /device/:key/toggle     Toggle device");
    info!("   - POST /device/:key/position   Set blind position");
    info!("   - GET  /health                 Health check");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "KNX-HomeKit Bridge API v1.0"
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

async fn list_devices(State(state): State<ApiState>) -> impl IntoResponse {
    let devices = state.state_manager.get_all_devices().await;

    let filtered_devices: Vec<DeviceInfo> = devices
        .iter()
        .filter(|d| !should_filter_device(d))
        .map(DeviceInfo::from)
        .collect();

    let total = filtered_devices.len();

    (
        StatusCode::OK,
        Json(DeviceListResponse {
            devices: filtered_devices,
            total,
        }),
    )
}

fn should_filter_device(_device: &Device) -> bool {
    false
}

async fn get_device(
    State(state): State<ApiState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    match state.state_manager.get_device(&key).await {
        Some(device) => {
            let info = DeviceInfo::from(&device);
            (StatusCode::OK, Json(info)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Device not found: {key}"),
            }),
        )
            .into_response(),
    }
}

async fn get_device_state(
    State(state): State<ApiState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    match state.state_manager.get_device(&key).await {
        Some(device) => {
            let info = DeviceInfo::from(&device);
            (StatusCode::OK, Json(info.state)).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Device not found: {key}"),
            }),
        )
            .into_response(),
    }
}

async fn toggle_device(
    State(state): State<ApiState>,
    Path(key): Path<String>,
    Json(payload): Json<ToggleRequest>,
) -> impl IntoResponse {
    info!("API: Toggle request for {} to {}", key, payload.on);

    match state.state_manager.toggle_device(&key, payload.on).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "ok", "device": key, "on": payload.on})),
        )
            .into_response(),
        Err(e) => {
            warn!("API: Failed to toggle device {}: {}", key, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to toggle device: {}", e),
                }),
            )
                .into_response()
        }
    }
}

async fn set_blind_position(
    State(state): State<ApiState>,
    Path(key): Path<String>,
    Json(payload): Json<BlindPositionRequest>,
) -> impl IntoResponse {
    info!("API: Blind position request for {} to {}%", key, payload.position);

    match state.state_manager.set_blind_position(&key, payload.position).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({"status": "ok", "device": key, "position": payload.position})),
        )
            .into_response(),
        Err(e) => {
            warn!("API: Failed to set blind position {}: {}", key, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to set blind position: {}", e),
                }),
            )
                .into_response()
        }
    }
}
