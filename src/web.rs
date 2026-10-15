use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::model::{Config, Export, Point, PointInput, Settings, normalize};
use crate::outstation::{add_point, remove_point, restart, write_point};
use crate::state::{App, Shared};

const INDEX: &str = include_str!("../static/index.html");
const TEMPLATE: &str = include_str!("../static/import-template.json");

struct ApiError(String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(json!({ "error": self.0 }))).into_response()
    }
}

type ApiResult = Result<Json<Value>, ApiError>;

pub fn router(shared: Shared) -> Router {
    Router::new()
        .route("/", get(|| async { Html(INDEX) }))
        .route("/api/state", get(state))
        .route("/api/settings", post(settings))
        .route("/api/points", post(create))
        .route("/api/points/:id", put(update).delete(remove))
        .route("/api/points/:id/value", post(set_value))
        .route("/api/points/:id/flags", post(set_flags))
        .route("/api/log", delete(clear_log))
        .route("/api/config/export", get(export_config))
        .route("/api/config/import", post(import_config))
        .route("/api/config/template", get(import_template))
        .with_state(shared)
}

fn snapshot(app: &App) -> Value {
    let mut points = app.config.points.clone();
    points.sort_by_key(|p| (p.data.kind as u8, p.data.index));
    json!({
        "settings": app.config.settings,
        "points": points,
        "connected": app.connected,
        "running": app.runtime.is_some(),
        "error": app.error,
        "log": app.log,
    })
}

async fn state(State(shared): State<Shared>) -> Json<Value> {
    Json(snapshot(&shared.lock()))
}

fn ensure_unique(app: &App, input: &PointInput, except: Option<u64>) -> Result<(), ApiError> {
    match app.config.conflict(input, except) {
        Some(conflict) => Err(ApiError(conflict)),
        None => Ok(()),
    }
}

fn sync_runtime(app: &App, old: Option<&Point>, new: Option<&Point>) {
    if let Some(rt) = &app.runtime {
        rt.outstation.transaction(|db| {
            if let Some(old) = old {
                remove_point(db, old.data.kind, old.data.index);
            }
            if let Some(new) = new {
                add_point(db, new);
                write_point(db, new);
            }
        });
    }
}

async fn create(State(shared): State<Shared>, Json(mut input): Json<PointInput>) -> ApiResult {
    input.validate().map_err(ApiError)?;
    let mut app = shared.lock();
    ensure_unique(&app, &input, None)?;
    let point = Point {
        id: app.config.next_id,
        data: input,
    };
    app.config.next_id += 1;
    sync_runtime(&app, None, Some(&point));
    app.log(
        true,
        format!(
            "Tambah {}[{}] \"{}\"",
            point.data.kind.label(),
            point.data.index,
            point.data.name
        ),
    );
    app.config.points.push(point);
    app.persist();
    Ok(Json(snapshot(&app)))
}

async fn update(
    State(shared): State<Shared>,
    Path(id): Path<u64>,
    Json(mut input): Json<PointInput>,
) -> ApiResult {
    input.validate().map_err(ApiError)?;
    let mut app = shared.lock();
    ensure_unique(&app, &input, Some(id))?;
    let position = app
        .config
        .points
        .iter()
        .position(|p| p.id == id)
        .ok_or(ApiError("point tidak ditemukan".into()))?;
    let old = app.config.points[position].clone();
    let new = Point { id, data: input };
    sync_runtime(&app, Some(&old), Some(&new));
    app.log(
        true,
        format!(
            "Ubah {}[{}] → {}[{}] \"{}\"",
            old.data.kind.label(),
            old.data.index,
            new.data.kind.label(),
            new.data.index,
            new.data.name
        ),
    );
    app.config.points[position] = new;
    app.persist();
    Ok(Json(snapshot(&app)))
}

async fn remove(State(shared): State<Shared>, Path(id): Path<u64>) -> ApiResult {
    let mut app = shared.lock();
    let position = app
        .config
        .points
        .iter()
        .position(|p| p.id == id)
        .ok_or(ApiError("point tidak ditemukan".into()))?;
    let old = app.config.points.remove(position);
    sync_runtime(&app, Some(&old), None);
    app.log(
        true,
        format!(
            "Hapus {}[{}] \"{}\"",
            old.data.kind.label(),
            old.data.index,
            old.data.name
        ),
    );
    app.persist();
    Ok(Json(snapshot(&app)))
}

#[derive(Deserialize)]
struct ValueInput {
    value: f64,
}

async fn set_value(
    State(shared): State<Shared>,
    Path(id): Path<u64>,
    Json(input): Json<ValueInput>,
) -> ApiResult {
    let mut app = shared.lock();
    let point = app
        .config
        .points
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or(ApiError("point tidak ditemukan".into()))?;
    let value = normalize(point.data.kind, input.value).map_err(ApiError)?;
    app.set_value(point.data.kind, point.data.index, value);
    app.log(
        true,
        format!(
            "Manual {}[{}] = {value}",
            point.data.kind.label(),
            point.data.index
        ),
    );
    app.persist();
    Ok(Json(snapshot(&app)))
}

#[derive(Deserialize)]
struct FlagsInput {
    flags: u8,
}

async fn set_flags(
    State(shared): State<Shared>,
    Path(id): Path<u64>,
    Json(input): Json<FlagsInput>,
) -> ApiResult {
    let mut app = shared.lock();
    let point = app
        .config
        .points
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or(ApiError("point tidak ditemukan".into()))?;
    point.data.flags = input.flags;
    let point = point.clone();
    app.write(std::slice::from_ref(&point));
    app.log(
        true,
        format!(
            "Quality {}[{}] = 0x{:02X}",
            point.data.kind.label(),
            point.data.index,
            input.flags
        ),
    );
    app.persist();
    Ok(Json(snapshot(&app)))
}

async fn clear_log(State(shared): State<Shared>) -> Json<Value> {
    let mut app = shared.lock();
    app.log.clear();
    Json(snapshot(&app))
}

async fn settings(State(shared): State<Shared>, Json(input): Json<Settings>) -> ApiResult {
    input.validate().map_err(ApiError)?;
    {
        let mut app = shared.lock();
        app.config.settings = input;
        app.persist();
    }
    apply_restart(&shared).await
}

fn download(filename: &str, body: String) -> Response {
    (
        [
            (
                header::CONTENT_TYPE,
                "application/json; charset=utf-8".to_string(),
            ),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        body,
    )
        .into_response()
}

async fn export_config(State(shared): State<Shared>) -> Result<Response, ApiError> {
    let export = shared.lock().config.export();
    let body = serde_json::to_string_pretty(&export).map_err(|e| ApiError(e.to_string()))?;
    Ok(download("simulator-rtu-dnp3.json", body + "\n"))
}

async fn import_template() -> Response {
    download(
        "template-import-simulator-rtu-dnp3.json",
        TEMPLATE.to_string(),
    )
}

async fn import_config(State(shared): State<Shared>, Json(input): Json<Export>) -> ApiResult {
    let config = Config::import(input).map_err(ApiError)?;
    {
        let mut app = shared.lock();
        let count = config.points.len();
        app.config = config;
        app.persist();
        app.log(true, format!("Import konfigurasi: {count} point"));
    }
    apply_restart(&shared).await
}

async fn apply_restart(shared: &Shared) -> ApiResult {
    let result = restart(shared).await;
    let mut app = shared.lock();
    if let Err(err) = result {
        app.error = Some(err.clone());
        app.log(false, err.clone());
        return Err(ApiError(err));
    }
    Ok(Json(snapshot(&app)))
}
