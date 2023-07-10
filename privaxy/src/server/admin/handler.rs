use crate::{blocker::BlockingDisabledStore, statistics::Statistics};
use http::{Method, Request, Response, StatusCode};
use hyper::{Body, Error};
use serde_json::{Value, json};

pub(crate) async fn handle_admin_request(req: Request<Body>, statistics: Statistics, blocking_store: BlockingDisabledStore) -> Result<Response<Body>, hyper::Error> {
    log::info!("{:#?} - {} {}", req.version(), req.method(), req.uri());
    
    if req.uri().path().starts_with("/api/") {
        let mut status_code: StatusCode = StatusCode::OK;
        let body: Value;
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/api/statistics") => {
                body = json!(statistics.get_serialized());
            },
            (&Method::GET, "/api/blocking") => {
                let state = blocking_store.is_enabled();
                body = json!({"state": match state { true => "Enabled", false => "Disabled" } });
            },
            (&Method::POST, "/api/blocking") => {
                let data: Value = serde_json::from_slice(hyper::body::to_bytes(req.into_body()).await.expect("valid byte stream").as_ref()).expect("valid json");
                let enabled = data.get("state");
                let mut passed = true;
                match enabled {
                    Some(x) => {
                        let x = x.as_str().expect("valid string?");
                        match x {
                            "Enabled" => blocking_store.set(true),
                            "Disabled" => blocking_store.set(false),
                            _ => passed = false,
                        }
                    },
                    None => passed = false,
                }
                if passed {
                    status_code = StatusCode::OK;
                    body = json!({"state":enabled});
                } else {
                    status_code = StatusCode::BAD_REQUEST;
                    body = json!({});
                }
            },
            (_, _) => {
                status_code = StatusCode::NOT_FOUND;
                body = json!({"status":404,"message":"Not Found"})
            }
        };
        return Ok::<_, Error>(
            Response::builder()
                .status(status_code)
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        );
    } else {
        let mut run_exe = std::env::current_exe().unwrap().clone();
        run_exe.pop();
        run_exe.push("dist");

        if req.uri().path() == "/" {
            run_exe.push("index.html");
        } else {
            run_exe.push(&(req.uri().path())[1..]);
        }
        let path_clone = run_exe.clone();

        if let Ok(contents) = tokio::fs::read(&run_exe.clone()).await {
            let mime_type = get_mime_type(path_clone.to_str().unwrap());
            // let body = contents.into();
            return Ok::<_, Error>(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime_type)
                    .body(Body::from(contents))
                    .unwrap(),
            );
        }
        Ok::<_, Error>(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/plain")
                .body(Body::from("Not Found"))
                .unwrap(),
        )
    }
}

fn get_mime_type(file: &str) -> &str {
    match file.split(".").last() {
        Some(v) => match v {
            "png" => "image",
            "jpg" => "image/jpeg",
            "json" => "application/json",
            "js" => "text/javascript",
            "html" => "text/html",
            "css" => "text/css",
            "wasm" => "application/wasm",
            &_ => "text/plain",
        },
        None => "text/plain",
    }
}
