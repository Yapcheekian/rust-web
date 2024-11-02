pub mod task_rpc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use task_rpc::{create_task, delete_task, list_tasks, update_task};

use crate::{
    ctx::Ctx,
    model::ModelManager,
    web::{Error, Result},
};

use crate::web::mw_auth::CtxW;

#[derive(Deserialize)]
struct RpcRequest {
    id: Option<String>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
    id: i64,
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsIded {
    id: i64,
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm)
}

pub async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx_w: CtxW,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    _rpc_handler(mm, ctx_w.0, rpc_req).await.into_response()
}

macro_rules! exec_rpc_fn {
    ($rpc_fn:expr, $ctx:expr, $mm:expr) => {
        $rpc_fn($ctx, $mm).await.map(to_value)??
    };

    ($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_param:expr) => {{
        let rpc_fn_name = stringify!($rpc_fn);

        let params = $rpc_param.ok_or(Error::RpcMissingParams {
            method: rpc_fn_name.to_string(),
        })?;

        let params = from_value(params).map_err(|_| Error::RpcFailJsonParams {
            method: rpc_fn_name.to_string(),
        })?;

        $rpc_fn($ctx, $mm, params).await.map(to_value)??
    }};
}

pub async fn _rpc_handler(mm: ModelManager, ctx: Ctx, rpc_req: RpcRequest) -> Result<Json<Value>> {
    let RpcRequest {
        id: rpc_id,
        method,
        params,
    } = rpc_req;

    let result_json = match method.as_str() {
        "create_task" => exec_rpc_fn!(create_task, ctx, mm, params),
        "list_tasks" => exec_rpc_fn!(list_tasks, ctx, mm),
        "update_task" => exec_rpc_fn!(update_task, ctx, mm, params),
        "delete_task" => exec_rpc_fn!(delete_task, ctx, mm, params),
        _ => return Err(Error::RpcMethodUnknown(method)),
    };

    let body_response = json!({
        "id": rpc_id,
        "result": result_json,
    });

    Ok(Json(body_response))
}
