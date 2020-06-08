use diesel::{self, prelude::*};
use warp::{
    http::Method,
    Filter,
    path
};
use crate::settings::{SETTINGS, Settings, JSON_BODY_LIMIT};
use crate::reject::handle_rejection;
use crate::db::{pg_pool, PgPool};
use crate::{async_clone_fn, async_clone_cb};
use crate::templates::register_templates;
use crate::reject::{CustomWarpRejection, RequiredData};
use super::cors::get_cors;
use std::net::SocketAddr;
use crate::templates::{
    direct::{DirectPage, direct_template},
    spa::{SpaPage, spa_template},
    epoch::epoch_page
};
use crate::user::auth::has_auth;

//All of our routes
pub fn get_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {

    let pool = pg_pool();

    let hb = register_templates();
    

    path::end()
        .and_then({ 
            let hb = hb.clone(); 
            move || direct_template(hb.clone(), DirectPage::Home)
        })
        .or(path!("no-auth").and_then({ 
            let hb = hb.clone(); 
            move || direct_template(hb.clone(), DirectPage::NoAuth)
        }))
        .or(path!("user" / "profile")
            .and(has_auth(None))
            .and_then({ 
                let hb = hb.clone(); 
                move |_| spa_template(hb.clone(), SpaPage::User)
            })
        )
        .or(path!("user" /..).and_then({ 
            let hb = hb.clone(); 
            move || spa_template(hb.clone(), SpaPage::User)
        }))
        .or(warp::fs::dir("./public/"))
        .or(path!("epoch").map(epoch_page))
        .recover(move |rej| handle_rejection(hb.clone(), rej))
        .with(get_cors())
}

//Decode the body as a specific json type
//and limit the length to prevent DoS
fn json_body_limit<T: serde::de::DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(JSON_BODY_LIMIT).and(warp::body::json())
}