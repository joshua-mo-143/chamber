use boulder_server::router::init_router;
use boulder_server::state::AppState;
mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, HttpBody},
        http::{self, Request, StatusCode},
    };

    use std::net::SocketAddr;
    use std::net::TcpListener;
    use tower::ServiceExt;
    use nanoid::nanoid;
    use serde_json::Value;

    #[tokio::test]
    async fn hello_world() {
        let state = AppState::new();
        let app = init_router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::LOCKED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"The vault is locked!");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn unseal_works() {
        let state = AppState::new();
        let app = init_router(state.clone());

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        let client = hyper::Client::new();

        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .uri(format!("http://{}/unseal", addr))
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({"key": state.db.sealkey})).unwrap(),
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn create_user() {
        let state = AppState::new();
        let app = init_router(state.clone());

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        let jwt_key = common::create_user_and_log_in(addr, &state.db.sealkey).await;

        let client = hyper::Client::new();
   }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn creating_a_secret_works() {
        let state = AppState::new();
        let app = init_router(state.clone());

        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        let jwt_key = common::create_user_and_log_in(addr, &state.db.sealkey).await;

        println!("{jwt_key}");

        let client = hyper::Client::new();

        let response = client
            .request(
                Request::builder()
                    .header("Authorization", &jwt_key)
                    .header("Content-Type", "application/json")
                    .uri(format!("http://{}/secrets/set", addr))
                    .method(http::Method::POST)
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({"key": "hello_world", "value":"meme"})).unwrap(),
                            ))
                    .unwrap(),
            )
            .await
            .unwrap();
    
        assert_eq!(response.status(), StatusCode::CREATED);

        let response = client
            .request(
                Request::builder()
                    .header("Authorization", &jwt_key)
                    .header("Content-Type", "application/json")
                    .uri(format!("http://{}/secrets/get", addr))
                    .method(http::Method::POST)
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({"key": "hello_world"})).unwrap(),
                            ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = std::str::from_utf8(&body).unwrap();
        assert_eq!(body, "meme");
   }
}
