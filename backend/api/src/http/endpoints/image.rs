use crate::{
    db::{self, meta::MetaWrapperError, nul_if_empty},
    error::{self, ServiceKind},
    extractor::{AuthUserWithScope, ScopeManageImage, WrapAuthClaimsNoDb},
    image_ops::generate_images,
    s3,
};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Bytes, Data, Json, Path, PayloadConfig, Query, ServiceConfig},
    CreatedJson, NoContent,
};
use shared::{
    api::{endpoints, ApiEndpoint},
    domain::{
        image::{
            CreateResponse, Image, ImageId, ImageKind, ImageResponse, ImageSearchResponse,
            ImageUpdateRequest,
        },
        meta::MetaKind,
    },
    media::{FileKind, MediaLibrary, PngImageFile},
};
use sqlx::{postgres::PgDatabaseError, PgPool};
use uuid::Uuid;

pub mod user {
    use crate::{db, error, extractor::WrapAuthClaimsNoDb, image_ops::generate_images, s3};
    use paperclip::actix::{
        api_v2_operation,
        web::{Bytes, Data, Json, Path},
        CreatedJson, NoContent,
    };

    use futures::TryStreamExt;
    use shared::{
        api::{endpoints, ApiEndpoint},
        domain::{
            image::{
                user::{UserImage, UserImageListResponse, UserImageResponse},
                ImageId, ImageKind,
            },
            CreateResponse,
        },
        media::MediaLibrary,
        media::{FileKind, PngImageFile},
    };
    use sqlx::PgPool;

    /// Create a image in the user's image library.
    #[api_v2_operation]
    pub(super) async fn create(
        db: Data<PgPool>,
        _claims: WrapAuthClaimsNoDb,
    ) -> Result<CreatedJson<<endpoints::image::user::Create as ApiEndpoint>::Res>, error::Server>
    {
        let id = db::image::user::create(db.as_ref()).await?;
        Ok(CreatedJson(CreateResponse { id }))
    }

    /// Upload an image to the user's image library.
    #[api_v2_operation]
    pub(super) async fn upload(
        db: Data<PgPool>,
        s3: Data<s3::Client>,
        _claims: WrapAuthClaimsNoDb,
        Path(id): Path<ImageId>,
        bytes: Bytes,
    ) -> Result<NoContent, error::Upload> {
        let mut txn = db.begin().await?;

        sqlx::query!(
            r#"select 1 as discard from user_image_library where id = $1 for update"#,
            id.0
        )
        .fetch_optional(&mut txn)
        .await?
        .ok_or(error::Upload::ResourceNotFound)?;

        let kind = ImageKind::Sticker;

        let (original, resized, thumbnail) =
            actix_web::web::block(move || -> Result<_, error::Upload> {
                let original =
                    image::load_from_memory(&bytes).map_err(|_| error::Upload::InvalidMedia)?;
                Ok(generate_images(&original, kind)?)
            })
            .await
            .map_err(error::Upload::blocking_error)?;

        s3.upload_png_images(MediaLibrary::User, id.0, original, resized, thumbnail)
            .await?;

        sqlx::query!(
            "update user_image_library set uploaded_at = now() where id = $1",
            id.0
        )
        .execute(&mut txn)
        .await?;

        txn.commit().await?;

        Ok(NoContent)
    }

    /// Delete an image from the user's image library.
    #[api_v2_operation]
    pub(super) async fn delete(
        db: Data<PgPool>,
        _claims: WrapAuthClaimsNoDb,
        req: Path<ImageId>,
        s3: Data<s3::Client>,
    ) -> Result<NoContent, error::Delete> {
        let image = req.into_inner();
        db::image::user::delete(&db, image)
            .await
            .map_err(super::check_conflict_delete)?;

        let delete = |kind| s3.delete_media(MediaLibrary::User, FileKind::ImagePng(kind), image.0);
        let ((), (), ()) = futures::future::join3(
            delete(PngImageFile::Original),
            delete(PngImageFile::Resized),
            delete(PngImageFile::Thumbnail),
        )
        .await;

        Ok(NoContent)
    }

    /// Get an image from the user's image library.
    #[api_v2_operation]
    pub(super) async fn get(
        db: Data<PgPool>,
        _claims: WrapAuthClaimsNoDb,
        req: Path<ImageId>,
    ) -> Result<Json<<endpoints::image::user::Get as ApiEndpoint>::Res>, error::NotFound> {
        let metadata = db::image::user::get(&db, req.into_inner())
            .await?
            .ok_or(error::NotFound::ResourceNotFound)?;

        Ok(Json(UserImageResponse { metadata }))
    }

    /// List images from the user's image library.
    #[api_v2_operation]
    pub(super) async fn list(
        db: Data<PgPool>,
        _claims: WrapAuthClaimsNoDb,
    ) -> Result<Json<<endpoints::image::user::List as ApiEndpoint>::Res>, error::Server> {
        let images: Vec<_> = db::image::user::list(db.as_ref())
            .err_into::<error::Server>()
            .and_then(|metadata: UserImage| async { Ok(UserImageResponse { metadata }) })
            .try_collect()
            .await?;

        Ok(Json(UserImageListResponse { images }))
    }
}

// attempts to grab a uuid out of a string in the shape:
// Key (<key>)=(<uuid>)<postfix>
fn extract_uuid(s: &str) -> Option<Uuid> {
    // <uuid>)<postfix)
    let s = s.split('(').nth(2)?;
    let s = &s[0..s.find(')')?];
    s.parse().ok()
}

fn handle_metadata_err(err: sqlx::Error) -> MetaWrapperError {
    let db_err = match &err {
        sqlx::Error::Database(e) => e.downcast_ref::<PgDatabaseError>(),
        _ => return MetaWrapperError::Sqlx(err),
    };

    let id = db_err.detail().and_then(extract_uuid);

    match db_err.constraint() {
        Some("image_affiliation_affiliation_id_fkey") => MetaWrapperError::MissingMetadata {
            id,
            kind: MetaKind::Affiliation,
        },

        Some("image_age_range_age_range_id_fkey") => MetaWrapperError::MissingMetadata {
            id,
            kind: MetaKind::AgeRange,
        },

        Some("image_style_style_id_fkey") => MetaWrapperError::MissingMetadata {
            id,
            kind: MetaKind::Style,
        },

        Some("image_category_category_id_fkey") => MetaWrapperError::MissingMetadata {
            id,
            kind: MetaKind::Category,
        },

        _ => MetaWrapperError::Sqlx(err),
    }
}

/// Create an image in the global image library.
#[api_v2_operation]
async fn create(
    db: Data<PgPool>,
    _claims: AuthUserWithScope<ScopeManageImage>,
    req: Json<<endpoints::image::Create as ApiEndpoint>::Req>,
) -> Result<CreatedJson<<endpoints::image::Create as ApiEndpoint>::Res>, error::CreateWithMetadata>
{
    let req = req.into_inner();

    let mut txn = db.begin().await?;
    let id = db::image::create(
        &mut txn,
        &req.name,
        &req.description,
        req.is_premium,
        req.publish_at.map(DateTime::<Utc>::from),
        req.kind,
    )
    .await?;

    db::image::update_metadata(
        &mut txn,
        id,
        nul_if_empty(&req.affiliations),
        nul_if_empty(&req.age_ranges),
        nul_if_empty(&req.styles),
        nul_if_empty(&req.categories),
    )
    .await
    .map_err(handle_metadata_err)?;

    txn.commit().await?;

    Ok(CreatedJson(CreateResponse { id }))
}

/// Upload an image to the global image library.
#[api_v2_operation]
async fn upload(
    db: Data<PgPool>,
    s3: Data<s3::Client>,
    _claims: AuthUserWithScope<ScopeManageImage>,
    Path(id): Path<ImageId>,
    bytes: Bytes,
) -> Result<NoContent, error::Upload> {
    let mut txn = db.begin().await?;

    let kind = sqlx::query!(
        r#"select kind as "kind: ImageKind" from image_metadata where id = $1 for update"#,
        id.0
    )
    .fetch_optional(&mut txn)
    .await?
    .ok_or(error::Upload::ResourceNotFound)?
    .kind;

    let (original, resized, thumbnail) =
        actix_web::web::block(move || -> Result<_, error::Upload> {
            let original =
                image::load_from_memory(&bytes).map_err(|_| error::Upload::InvalidMedia)?;
            Ok(generate_images(&original, kind)?)
        })
        .await
        .map_err(error::Upload::blocking_error)?;

    s3.upload_png_images(MediaLibrary::Global, id.0, original, resized, thumbnail)
        .await?;

    sqlx::query!(
        "update image_metadata set uploaded_at = now() where id = $1",
        id.0
    )
    .execute(&mut txn)
    .await?;

    txn.commit().await?;

    Ok(NoContent)
}

/// Get an image from the global image library.
#[api_v2_operation]
async fn get_one(
    db: Data<PgPool>,
    _claims: WrapAuthClaimsNoDb,
    req: Path<ImageId>,
) -> Result<Json<<endpoints::image::Get as ApiEndpoint>::Res>, error::NotFound> {
    let metadata = db::image::get_one(&db, req.into_inner())
        .await?
        .ok_or(error::NotFound::ResourceNotFound)?;

    Ok(Json(ImageResponse { metadata }))
}

/// Search for images in the global image library.
#[api_v2_operation]
async fn search(
    db: Data<PgPool>,
    algolia: Data<crate::algolia::Client>,
    _claims: WrapAuthClaimsNoDb,
    query: Option<Query<<endpoints::image::Search as ApiEndpoint>::Req>>,
) -> Result<Json<<endpoints::image::Search as ApiEndpoint>::Res>, error::Service> {
    let query = dbg!(query.map_or_else(Default::default, Query::into_inner));

    let (ids, pages, total_hits) = algolia
        .search_image(
            &query.q,
            query.page,
            query.is_premium,
            query.is_published,
            &query.styles,
            &query.age_ranges,
            &query.affiliations,
            &query.categories,
        )
        .await?
        .ok_or_else(|| error::Service::DisabledService(ServiceKind::Algolia))?;

    let images: Vec<_> = db::image::get(db.as_ref(), &ids)
        .err_into::<error::Service>()
        .and_then(|metadata: Image| async { Ok(ImageResponse { metadata }) })
        .try_collect()
        .await?;

    Ok(Json(ImageSearchResponse {
        images,
        pages,
        total_image_count: total_hits,
    }))
}

/// Update an image in the global image library.
#[api_v2_operation]
async fn update(
    db: Data<PgPool>,
    _claims: AuthUserWithScope<ScopeManageImage>,
    req: Option<Json<<endpoints::image::UpdateMetadata as ApiEndpoint>::Req>>,
    id: Path<ImageId>,
) -> Result<NoContent, error::UpdateWithMetadata> {
    let req = req.map_or_else(ImageUpdateRequest::default, Json::into_inner);
    let id = id.into_inner();
    let mut txn = db.begin().await?;

    let exists = db::image::update(
        &mut txn,
        id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.is_premium,
        req.publish_at.map(|it| it.map(DateTime::<Utc>::from)),
    )
    .await?;

    if !exists {
        return Err(error::UpdateWithMetadata::ResourceNotFound);
    }

    db::image::update_metadata(
        &mut txn,
        id,
        req.affiliations.as_deref(),
        req.age_ranges.as_deref(),
        req.styles.as_deref(),
        req.categories.as_deref(),
    )
    .await
    .map_err(handle_metadata_err)?;

    txn.commit().await?;

    Ok(NoContent)
}

fn check_conflict_delete(err: sqlx::Error) -> error::Delete {
    match err {
        sqlx::Error::Database(e) if e.downcast_ref::<PgDatabaseError>().constraint().is_some() => {
            error::Delete::Conflict
        }
        _ => error::Delete::InternalServerError(err.into()),
    }
}

/// Delete an image from the global image library.
#[api_v2_operation]
async fn delete(
    db: Data<PgPool>,
    algolia: Data<crate::algolia::Client>,
    _claims: AuthUserWithScope<ScopeManageImage>,
    req: Path<ImageId>,
    s3: Data<s3::Client>,
) -> Result<NoContent, error::Delete> {
    let image = req.into_inner();
    db::image::delete(&db, image)
        .await
        .map_err(check_conflict_delete)?;

    // todo: 501 when algolia is disabled.

    let delete = |kind| s3.delete_media(MediaLibrary::Global, FileKind::ImagePng(kind), image.0);
    let ((), (), (), ()) = futures::future::join4(
        delete(PngImageFile::Original),
        delete(PngImageFile::Resized),
        delete(PngImageFile::Thumbnail),
        algolia.delete_image(image),
    )
    .await;

    Ok(NoContent)
}

pub fn configure(cfg: &mut ServiceConfig<'_>) {
    use endpoints::image;
    cfg.route(
        image::Create::PATH,
        image::Create::METHOD.route().to(create),
    )
    .service(
        web::resource(image::Upload::PATH)
            .app_data(PayloadConfig::default().limit(config::IMAGE_BODY_SIZE_LIMIT))
            .route(image::Upload::METHOD.route().to(upload)),
    )
    .route(image::Get::PATH, image::Get::METHOD.route().to(get_one))
    .route(
        image::Search::PATH,
        image::Search::METHOD.route().to(search),
    )
    .route(
        image::UpdateMetadata::PATH,
        image::UpdateMetadata::METHOD.route().to(update),
    )
    .route(
        image::Delete::PATH,
        image::Delete::METHOD.route().to(delete),
    )
    .route(
        image::user::Create::PATH,
        image::user::Create::METHOD.route().to(self::user::create),
    )
    .route(
        image::user::Upload::PATH,
        image::user::Upload::METHOD.route().to(self::user::upload),
    )
    .route(
        image::user::Delete::PATH,
        image::user::Delete::METHOD.route().to(self::user::delete),
    )
    .route(
        image::user::Get::PATH,
        image::user::Get::METHOD.route().to(self::user::get),
    )
    .route(
        image::user::List::PATH,
        image::user::List::METHOD.route().to(self::user::list),
    );
}
