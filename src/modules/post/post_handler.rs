use crate::{midleware::authmiddlewares:: Authentication, AppState};
use super::post_models::{NewPost,Post};
use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use sqlx::query_as;
use uuid::Uuid;

#[get("/getall/{page}")]
pub async fn get_all_post(
    path:web::Path<i64>,
    data: web::Data<AppState>,
) -> impl Responder {
    let page = path.into_inner();
    let limit:i64 = 10;
    let offset = (page - 1)*limit;
    let posts = query_as!(
        Post,
        r#"SELECT * FROM post ORDER BY id LIMIT $1 OFFSET $2"#,
        limit,
        offset,
    ).fetch_all(&data.db).await;
    match posts {
        Ok(posts) => {
            let json_response = serde_json::json!({
                "status": "ok",
                "data": posts,
            });
            HttpResponse::Ok().json(json_response)
        },
        Err(_) => {
            let message = "Something bad happened when fetching all posts";
            HttpResponse::InternalServerError().json(
                serde_json::json!({"status": "error", "message": message}),
            )
        },
    }
}

#[post("")]
async fn create_post_handlers(
    mut body:web::Json<NewPost>,
    data:web::Data<AppState>,
    req:HttpRequest
) -> impl Responder {
    match req.extensions().get::<Uuid>().cloned(){
        Some(id)=>body.user_id = Some(id),
        None=> return HttpResponse::Unauthorized().json(json!({"message":"invalid user id","status":"fail"}))
    }
    let new_post = query_as!(
        Post,
        r#"INSERT INTO post(title, content,user_id) VALUES ($1, $2,$3) RETURNING *"#,
        body.title,
        body.content,
        body.user_id
    )
    .fetch_one(&data.db)
    .await;

    match new_post {
        Ok(post)=>{
            let response_json = serde_json::json!({"status":"success","data":serde_json::json!({
                "post":post
            })});

            return HttpResponse::Created().json(json!(response_json))
        }
        Err(e)=>{
            if e.to_string()
            .contains("duplicate key value violates unique constraint")
                {
                    return HttpResponse::BadRequest()
                    .json(serde_json::json!({"status": "fail","message": "Post with that title already exists"}));
                }

                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
                }
    }
}


pub fn post_config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/post")
        .wrap(Authentication)
        .service(get_all_post)
        .service(create_post_handlers);

    conf.service(scope);
}