use super::*;
    use axum::{
        body::{Body, HttpBody},
        http::{self, Request, StatusCode},
    };

    use std::net::SocketAddr;
    use std::net::TcpListener;
    use tower::ServiceExt;

use axum::{
    routing::{get, post},
    Router,
};
use nanoid::nanoid;
use serde_json::Value;
use boulder_server::secrets;
use boulder_server::state::AppState;

pub fn test_router() -> (Router, AppState) {
    let state = AppState::new();

    let router = Router::new()
        .route("/", get(boulder_server::router::hello_world))
        .route("/secrets/set", post(secrets::create_secret))
        .route("/secrets/get", post(secrets::view_secret))
        .route("/unseal", post(secrets::unlock))
        .with_state(state.clone());

    (router, state)
}

pub async fn start_router_unlocked(addr: SocketAddr, key: String) {
        let client = hyper::Client::new();

        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .uri(format!("http://{}/unseal", addr))
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({"key": key})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .request(
                Request::builder()
                    .uri(format!("http://{}", addr))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, world!");
}


pub async fn create_user_and_log_in(addr: SocketAddr, key: &str) -> String {
        let client = hyper::Client::new();

        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .uri(format!("http://{}/unseal", addr))
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({"key": key})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .request(
                Request::builder()
                    .uri(format!("http://{}", addr))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Hello, world!");

        let random_name = nanoid!(10);

        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .header("x-boulder-key", key)
                    .uri(format!("http://{}/users/create", addr))
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({"name": &random_name})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

       assert_eq!(response.status(), StatusCode::CREATED); 

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let password = std::str::from_utf8(&body).unwrap();

        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .uri(format!("http://{}/login", addr))
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({"password": &password})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

       assert_eq!(response.status(), StatusCode::OK); 

       let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Value = serde_json::from_slice(&body).unwrap();

        assert!(body.get("access_token").is_some());

        let res = format!("{} {}", body.get("token_type").unwrap(), body.get("access_token").unwrap());

        let new_str = &res.replace("\"", "");

        new_str.to_owned()
    
}

