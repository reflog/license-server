use serde::{Deserialize, Serialize};
use serde_json::json;
use warp::hyper::header::HeaderValue;
use warp::Filter;

use crate::model;
use std::convert::Infallible;
use warp::http::StatusCode;

#[derive(Debug)]
struct InvalidLicense;
impl warp::reject::Reject for InvalidLicense {}

#[derive(Debug)]
struct InvalidAPIKey;
impl warp::reject::Reject for InvalidAPIKey {}

#[derive(Debug)]
struct InvalidGenerateRequest;
impl warp::reject::Reject for InvalidGenerateRequest {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct EncodedLicense {
    pub(crate) license: String,
}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

pub(crate) async fn handle_validate(
    secret: String,
    encoded_license: EncodedLicense,
) -> Result<impl warp::Reply, warp::Rejection> {
    let sl = model::SignedLicense::new(&encoded_license.license)
        .map_err(|_| warp::reject::custom(InvalidLicense))?;
    sl.validate(secret)
        .map_err(|_| warp::reject::custom(InvalidLicense))?;
    let v = json!({ "result": "Valid!" });
    Ok(warp::reply::json(&v))
}

pub(crate) async fn handle_generate(
    secret: String,
    api_key: String,
    auth: HeaderValue,
    lic: model::License,
) -> Result<impl warp::Reply, warp::Rejection> {
    let auth_header = auth
        .to_str()
        .map_err(|_| warp::reject::custom(InvalidAPIKey))?;
    if auth_header != api_key {
        return Err(warp::reject::custom(InvalidAPIKey));
    }
    let mut l = lic.clone();
    l.id = Some(uuid::Uuid::new_v4().to_string());
    let hash = l
        .hash(secret)
        .map_err(|_| warp::reject::custom(InvalidGenerateRequest))?;
    let v = json!({ "result": hash });
    Ok(warp::reply::json(&v))
}

fn api_validate(
    secret: String,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("validate")
        .and(warp::post())
        .and(warp::any().map(move || secret.clone()))
        .and(warp::body::json())
        .and_then(handle_validate)
}

fn api_generate(
    secret: String,
    api_key: String,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("generate")
        .and(warp::post())
        .and(warp::any().map(move || secret.clone()))
        .and(warp::any().map(move || api_key.clone()))
        .and(warp::header::value("X-API-Key"))
        .and(warp::body::json())
        .and_then(handle_generate)
}

pub(crate) async fn serve(secret: String, api_key: String, port: u16) {
    let secret_copy = secret.clone();

    let generate_path = api_generate(secret, api_key);
    let validate_path = api_validate(secret_copy);
    println!("Listening on {:}", port);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(&[warp::http::Method::POST])
        .allow_headers(vec!["x-api-key", "content-type"]);

    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
    };
    let (_, serving) = warp::serve(
        validate_path
            .or(generate_path)
            .with(cors)
            .recover(handle_rejection),
    )
    .bind_with_graceful_shutdown(([0, 0, 0, 0], port), shutdown);
    tokio::select! {
        _ = serving => {},
    }
}

// This function receives a `Rejection` and tries to return a custom
// value, otherwise simply passes the rejection along.
async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;

    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND";
    } else if err.find::<InvalidLicense>().is_some() {
        code = StatusCode::BAD_REQUEST;
        message = "INVALID_LICENSE";
    } else if err.find::<InvalidGenerateRequest>().is_some() {
        code = StatusCode::BAD_REQUEST;
        message = "INVALID_GENERATE_REQUEST";
    } else if err.find::<InvalidAPIKey>().is_some() {
        code = StatusCode::BAD_REQUEST;
        message = "INVALID_API_KEY";
    } else if let Some(_e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        message = "BAD_REQUEST";
        code = StatusCode::BAD_REQUEST;
    } else if err.find::<warp::reject::MissingHeader>().is_some() {
        code = StatusCode::BAD_REQUEST;
        message = "MISSING_API_KEY_HEADER";
    } else {
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

#[cfg(test)]
mod tests {
    use crate::model;
    use crate::serve::{api_generate, api_validate, handle_rejection};
    use warp::http::StatusCode;
    use warp::test::request;
    use warp::Filter;
    const SECRET: &str = "SECRET";
    const KEY: &str = "KEY";
    const TEST_LICENSE_REQUEST: &str = r#"{"license":"eyJsaWNlbnNlIjp7ImlkIjoidGVzdCIsIm1ldGEiOnt9LCJ2YWxpZF9mcm9tIjoiMjAwMC0xLTEiLCJ2YWxpZF91bnRpbCI6IjMwMDAtMS0xIn0sInNpZ25hdHVyZSI6ImVhYzJkMjI2ZjA0NTFjMmQ5NTM2NzkxZDg2NDEyMjRhZWFmMjkwY2NmZjEzYWQxZDE0YmYxY2U2OGMyYzJmMmQifQ=="}"#;
    #[tokio::test]
    async fn test_generate() {
        let generate_path =
            api_generate(SECRET.to_string(), KEY.to_string()).recover(handle_rejection);
        let license = model::License {
            id: None,
            meta: Default::default(),
            valid_from: "2000-1-1".to_string(),
            valid_until: "3000-1-1".to_string(),
        };
        let bad_resp = request()
            .method("POST")
            .path("/generate")
            .json(&license)
            .reply(&generate_path)
            .await;
        assert_eq!(bad_resp.status(), StatusCode::BAD_REQUEST);

        let bad_resp = request()
            .method("POST")
            .path("/generate")
            .header("X-API-Key", KEY.to_owned() + "_bad")
            .json(&license)
            .reply(&generate_path)
            .await;
        assert_eq!(bad_resp.status(), StatusCode::BAD_REQUEST);

        let resp = request()
            .method("POST")
            .path("/generate")
            .header("X-API-Key", KEY)
            .json(&license)
            .reply(&generate_path)
            .await;
        // assert_eq!(resp.body(), TEST_LICENSE_RESPONSE);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_validate() {
        let validate_path = api_validate(SECRET.to_string()).recover(handle_rejection);
        let resp = request()
            .method("POST")
            .path("/validate")
            .body(&TEST_LICENSE_REQUEST)
            .reply(&validate_path)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}
