use axum::{
    body::{Body, HttpBody},
    http::{self, Request, StatusCode},
};
use serde_json::Value;
use std::net::SocketAddr;

pub mod postgres;

pub async fn create_user_and_log_in(addr: SocketAddr, key: &str) -> String {
    let client = hyper::Client::new();

    let response = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://{}/unseal", addr))
                .header("x-chamber-key", key)
                .header("Content-Type", "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    //        let body = response.into_body().collect().await.unwrap().to_bytes();
    //        let string = std::str::from_utf8(&body).unwrap();
    //        assert_eq!(string, "The vault is locked!");
    assert_eq!(response.status(), StatusCode::OK);

    let response = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .header("Content-Type", "application/json")
                .uri(format!("http://{}/login", addr))
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!(
                    {
                        "username": "root",
                        "password": "this"
                    }
                    ))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

   assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert!(body.get("access_token").is_some());

    let res = format!(
        "{} {}",
        body.get("token_type").unwrap(),
        body.get("access_token").unwrap()
    );

    let new_str = &res.replace('\"', "");

    new_str.to_owned()
}
