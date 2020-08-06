use shared::{
    auth::SigninSuccess,
    user::NoSuchUserError
};
use core::{
    routes::{Route, UserRoute},
    fetch::user::fetch_signin,
    storage,
};
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::{JsFuture, spawn_local, future_to_promise};
use crate::utils::firebase::get_firebase_signin_google;
use futures_signals::signal::{Mutable, Signal, SignalExt};
use dominator::clone;
use std::rc::Rc;
use super::SigninPage;
use futures::future::ready;

//temp
use futures::future::poll_fn;
use futures::task::{Context, Poll};
#[derive(Debug, Clone)]
pub enum SigninStatus {
    Busy,
    NoSuchUser,
}

fn do_success(page:&SigninPage, csrf:String) {
    storage::save_csrf_token(&csrf);
    dominator::routing::go_to_url( Route::User(UserRoute::Profile).into());

    ///generally speaking this kind of thing isn't necessary
    ///futures will just resolve and be dropped as part of the flow
    ///but because the oauth flow here opens a separate window
    ///it's more at risk to leave dangling Futures
    ///specifically, here, dangling futures which hold the Rc that holds it
    ///thereby creating a cycle, we need to break by cancelling that future
    ///see: https://github.com/jewish-interactive/ji-cloud/issues/78
    page.signin_loader.cancel();
}

pub async fn signin_google(page:Rc<SigninPage>) {


    let token_promise = unsafe { get_firebase_signin_google() };

    match JsFuture::from(token_promise).await {
        Ok(token) => {
            let token = token.as_string().unwrap_throw();
            let resp:Result<SigninSuccess, NoSuchUserError> = fetch_signin(&token).await;
            match resp {
                Ok(data) => do_success(&page, data.csrf),
                Err(_) => page.status.set(Some(SigninStatus::NoSuchUser))
            }
        },
        Err(_) => {
            page.status.set(None);
        }
    };
}

pub async fn signin_email(page:Rc<SigninPage>) {

    let refs = page.refs.borrow();
    let refs = refs.as_ref().unwrap_throw();
    let email = refs.get_email();
    let pw = refs.get_pw();
    log::info!("signin clicked! email: {} pw: {}", email, pw);
}
