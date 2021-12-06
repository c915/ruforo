// use crate::session::new_session;
use crate::frontend::TemplateToPubResponse;
use crate::orm::users;
use crate::orm::users::Entity as Users;
use crate::session;
use crate::session::{authenticate_by_cookie, MainData};
use crate::template::LoginTemplate;
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use sea_orm::{entity::*, query::*, DatabaseConnection, FromQueryResult, QueryFilter};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

async fn login(
    db: &DatabaseConnection,
    name_: &str,
    pass_: &str,
    my: &web::Data<MainData<'_>>,
) -> Result<i32, Error> {
    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        id: i32,
        password: String,
    }

    let select = Users::find()
        .select_only()
        .column(users::Column::Id)
        .column(users::Column::Password)
        .filter(users::Column::Name.eq(name_));

    let user = select
        .into_model::<SelectResult>()
        .one(db)
        .await
        .map_err(|e| {
            log::error!("Login: {}", e);
            error::ErrorInternalServerError("DB error")
        })?
        .ok_or_else(|| error::ErrorInternalServerError("user not found or bad password"))?;

    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    my.argon2
        .verify_password(pass_.as_bytes(), &parsed_hash)
        .map_err(|_| error::ErrorInternalServerError("user not found or bad password"))?;
    Ok(user.id)
}

#[post("/login")]
pub async fn login_post(
    session: actix_session::Session,
    form: web::Form<FormData>,
    my: web::Data<MainData<'_>>,
) -> Result<HttpResponse, Error> {
    // don't forget to sanitize kek and add error handling
    let user_id = login(&my.pool, &form.username, &form.password, &my).await?;

    let uuid = session::new_session(&my.pool, &my.cache.sessions, user_id)
        .await
        .map_err(|e| {
            log::error!("error {:?}", e);
            error::ErrorInternalServerError("DB error")
        })?;

    session
        .insert("logged_in", true)
        .map_err(|_| error::ErrorInternalServerError("middleware error"))?;

    session
        .insert("token", uuid)
        .map_err(|_| error::ErrorInternalServerError("middleware error"))?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/login")]
pub async fn login_get(
    cookies: actix_session::Session,
    my: web::Data<MainData<'_>>,
) -> Result<impl Responder, Error> {
    let mut tmpl = LoginTemplate {
        user_id: None,
        logged_in: false,
        username: None,
        token: None,
    };

    let session = authenticate_by_cookie(&my.cache.sessions, &cookies).await;
    let uuid;
    if session.is_some() {
        let session = session.unwrap();
        tmpl.user_id = Some(session.session.user_id);
        tmpl.logged_in = true;
        uuid = session.uuid.to_string();
        tmpl.token = Some(&uuid);
    }

    Ok(tmpl.to_pub_response())
}
