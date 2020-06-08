use warp::http::Method;
use crate::settings::{SETTINGS, CORS_ORIGINS};

pub fn get_cors() -> warp::filters::cors::Builder {
    let builder = warp::cors()
        .allow_methods(&[Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers(vec!["Authorization", "Content-Type", "X-CSRF"])
        .allow_credentials(true);

    if(SETTINGS.local_insecure) {
        builder.allow_any_origin()
    } else {
        builder.allow_origins(CORS_ORIGINS.clone())
    }
}