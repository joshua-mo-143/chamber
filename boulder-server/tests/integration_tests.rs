use boulder_core::kv::InMemoryDatabase;
use boulder_core::users::Role;
use boulder_server::router::init_router;
use boulder_server::state::DynDatabase;
mod common;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, HttpBody},
        http::{self, Request, StatusCode},
    };
    use nanoid::nanoid;
    use std::net::SocketAddr;
    use std::net::TcpListener;
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn hello_world() {
        let state = Arc::new(InMemoryDatabase::new()) as DynDatabase;
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
        let state = Arc::new(InMemoryDatabase::new()) as DynDatabase;
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
                    .header("x-boulder-key", &state.get_root_key())
                    .header("Content-Type", "application/json")
                    .uri(format!("http://{}/unseal", addr))
                    .body(Body::empty())
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
        let state = Arc::new(InMemoryDatabase::new()) as DynDatabase;
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

        let _jwt_key = common::create_user_and_log_in(addr, &state.get_root_key()).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn create_user_with_user_role() {
        let state = Arc::new(InMemoryDatabase::new()) as DynDatabase;
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

        let _jwt_key = common::create_user_and_log_in(addr, &state.get_root_key()).await;
        let client = hyper::Client::new();

        let random_name = nanoid!(10);
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .header("x-boulder-key", state.get_root_key())
                    .uri(format!("http://{}/users/create", addr))
                    .body(Body::from(
                        serde_json::to_vec(
                            &serde_json::json!({"name": &random_name, "role": Role::User}),
                        )
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn create_user_with_editor_role() {
        let state = Arc::new(InMemoryDatabase::new()) as DynDatabase;
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

        let _jwt_key = common::create_user_and_log_in(addr, &state.get_root_key()).await;
        let client = hyper::Client::new();

        let random_name = nanoid!(10);
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .header("x-boulder-key", state.get_root_key())
                    .uri(format!("http://{}/users/create", addr))
                    .body(Body::from(
                        serde_json::to_vec(
                            &serde_json::json!({"name": &random_name, "role": Role::Editor}),
                        )
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn create_user_with_almost_root_role() {
        let state = Arc::new(InMemoryDatabase::new()) as DynDatabase;
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

        let _jwt_key = common::create_user_and_log_in(addr, &state.get_root_key()).await;
        let client = hyper::Client::new();

        let random_name = nanoid!(10);
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .header("x-boulder-key", &state.get_root_key())
                    .uri(format!("http://{}/users/create", addr))
                    .body(Body::from(
                        serde_json::to_vec(
                            &serde_json::json!({"name": &random_name, "role": Role::AlmostRoot}),
                        )
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn create_user_with_root_role() {
        let state = Arc::new(InMemoryDatabase::new()) as DynDatabase;
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

        let _jwt_key = common::create_user_and_log_in(addr, &state.get_root_key()).await;
        let client = hyper::Client::new();

        let random_name = nanoid!(10);
        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .header("x-boulder-key", &state.get_root_key())
                    .uri(format!("http://{}/users/create", addr))
                    .body(Body::from(
                        serde_json::to_vec(
                            &serde_json::json!({"name": &random_name, "role": Role::Root}),
                        )
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn creating_a_secret_works() {
        let state = Arc::new(InMemoryDatabase::new()) as DynDatabase;
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

        let jwt_key = common::create_user_and_log_in(addr, &state.get_root_key()).await;

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
                        serde_json::to_vec(
                            &serde_json::json!({"key": "hello_world", "value":"meme"}),
                        )
                        .unwrap(),
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
