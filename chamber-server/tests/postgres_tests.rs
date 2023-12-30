use chamber_core::traits::AppState;
use chamber_core::traits::RegularAppState;
use chamber_server::router::init_router;

mod common;
const BOUNDARY: &str = "------------------------ea3bbcf87c101592";

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, HttpBody},
        http::{self, Request, StatusCode},
    };
    use std::net::SocketAddr;
    use std::net::TcpListener;

    use chamber_core::secrets::KeyFile;
    use std::io::Write;
    use tower::ServiceExt;

    #[tokio::test]
    async fn hello_world() {
        let pool = common::postgres::get_test_db_connection().await;
        let state = RegularAppState::new(pool);
        state.check_keyfile_exists();

        let app = init_router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::LOCKED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"The vault is locked!");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn create_user() {
        let pool = common::postgres::get_test_db_connection().await;
        let state = RegularAppState::new(pool);

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

        let _ =
            common::create_user_and_log_in(addr, state.get_keyfile().unwrap().unseal_key()).await;

        let test_user = "test_user";

        let client = hyper::Client::new();

        let response = client
            .request(
                Request::builder()
                    .method(http::Method::POST)
                    .header("Content-Type", "application/json")
                    .header("x-chamber-key", state.get_keyfile().unwrap().unseal_key())
                    .uri(format!("http://{}/users/create", addr))
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({"name": test_user})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let password = std::str::from_utf8(&body).unwrap();
        println!("{password}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn creating_a_secret_works() {
        let pool = common::postgres::get_test_db_connection().await;
        let state = RegularAppState::new(pool);

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

        let jwt_key =
            common::create_user_and_log_in(addr, state.get_keyfile().unwrap().unseal_key()).await;

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

        // assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = std::str::from_utf8(&body).unwrap();
        assert_eq!(body, "meme");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn create_secret_with_access_level() {
        let pool = common::postgres::get_test_db_connection().await;
        let state = RegularAppState::new(pool);

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

        let jwt_key =
            common::create_user_and_log_in(addr, state.get_keyfile().unwrap().unseal_key()).await;

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
                        serde_json::to_vec(&serde_json::json!({
                            "key": "test key",
                            "value":"test key",
                            "tags":["Test", "Key"],
                            "access_level":500
                        }))
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
                        serde_json::to_vec(&serde_json::json!({"key": "test key"})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = std::str::from_utf8(&body).unwrap();
        assert_eq!(body, "test key");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn create_secret_with_access_level_and_role() {
        let pool = common::postgres::get_test_db_connection().await;
        let state = RegularAppState::new(pool);

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

        let jwt_key =
            common::create_user_and_log_in(addr, state.get_keyfile().unwrap().unseal_key()).await;

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
                        serde_json::to_vec(&serde_json::json!({
                            "key": "stripe_test_key",
                            "value":"my_key",
                            "tags":["Test", "Key"],
                            "access_level":500,
                            "role_whitelist":["Engineer"]
                        }))
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
                        serde_json::to_vec(&serde_json::json!({"key": "stripe_test_key"})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn rekeying_works() {
        let pool = common::postgres::get_test_db_connection().await;
        let state = RegularAppState::new(pool);

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

        let jwt_key =
            common::create_user_and_log_in(addr, state.get_keyfile().unwrap().unseal_key()).await;

        println!("{jwt_key}");

        let client = hyper::Client::new();

    let keyfile = KeyFile::new();

    let encoded = bincode::serialize(&keyfile).unwrap();

    let mut data = Vec::new();
    write!(data, "--{}\r\n", BOUNDARY).unwrap();
    write!(data, "Content-Disposition: form-data; name=\"file\"; filename=\"chamber.bin\"\r\n").unwrap();
    write!(data, "Content-Type: \r\n").unwrap();
    write!(data, "\r\n").unwrap();

    data.write(encoded.as_slice()).unwrap();

    write!(data, "\r\n").unwrap(); // The key thing you are missing
    write!(data, "--{}--\r\n", BOUNDARY).unwrap();

        let response = client
            .request(
                Request::builder()
                    .header("Authorization", &jwt_key)
                    .header("Content-Type", &*format!("multipart/form-data; boundary={}", BOUNDARY))
                    .uri(format!("http://{}/binfile", addr))
                    .method(http::Method::POST)
                    .body(data.into())
                    .unwrap()
                    )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = client
            .request(
                Request::builder()
                    .header("Authorization", &jwt_key)
                    .header("Content-Type", "application/json")
                    .uri(format!("http://{}/secrets/get", addr))
                    .method(http::Method::POST)
                    .body(Body::from(
                        serde_json::to_vec(&serde_json::json!({"key": "test key"})).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body = std::str::from_utf8(&body).unwrap();
        assert_eq!(body, "test key");
    }
}
