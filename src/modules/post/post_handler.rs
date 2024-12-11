use crate::{midleware::authmiddlewares:: Authentication, AppState};
use super::post_models::{NewPost,Post,UpdatePost};
use actix_web::{delete, get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use sqlx::{query, query_as};
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

#[get("/detail/{id}")]
pub async fn get_one_post(
    path:web::Path<i32>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();
    let post = query_as!(
        Post,
        r#"SELECT * FROM post WHERE id=$1"#,
        id,
    ).fetch_one(&data.db).await;
    match post {
        Ok(posts) => {
            let json_response = serde_json::json!({
                "status": "ok",
                "data": posts,
            });
            HttpResponse::Ok().json(json_response)
        },
        Err(err) => {
            if err.to_string().contains("no rows returned by a query that expected to return at least one row") {
                return HttpResponse::NotFound().json(
                    json!({"status":"failed","messsage":"data not found"})
                )
            }
            else {
            HttpResponse::InternalServerError().json(
                serde_json::json!({"status": "error", "message": "Something bad happened when fetching all posts"}),
            )
            }
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
        None=> return HttpResponse::Unauthorized().json(json!({"message":"invalid user id","status":"failed"}))
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
                    .json(serde_json::json!({"status": "failed","message": "Post with that title already exists"}));
                }

                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
                }
    }
}

#[delete("/{id}")]
pub async fn delete_post_by_id(
    id:web::Path<i32>,
    data: web::Data<AppState>,
    req:HttpRequest
) -> impl Responder {
    if let Some(user_id) = req.extensions().get::<Uuid>().cloned(){
        let id = id.into_inner();
        let post = query_as!(
            Post,
            r#"SELECT * FROM post WHERE id = $1 AND user_id = $2"#,
            id,
            user_id
        ).fetch_one(&data.db).await;
    
        match post {
            Ok(post) => {
            query!("DELETE FROM post where id=$1",post.id).execute(&data.db).await.unwrap().rows_affected();
    
                let json_response = json!({
                    "status": "ok",
                    "message":"post delete success",
                });
                HttpResponse::Ok().json(json_response)
            },
            Err(err) => {
                if err.to_string().contains("no rows returned by a query that expected to return at least one row") {
                    return HttpResponse::NotFound().json(
                        json!({"status":"failed","messsage":"data not found"})
                    )
                }
                else {
                HttpResponse::InternalServerError().json(
                    serde_json::json!({"status": "error", "message": "Something bad happened when fetching all posts"}),
                )
                }
            },
        }
    
    }else{
        return HttpResponse::Unauthorized().json(json!({"message":"invalid user id","status":"failed"}))
    }
}

#[patch("/{id}")]
pub async fn update_post_by_id(
    id:web::Path<i32>,
    data: web::Data<AppState>,
    body:web::Json<UpdatePost>,
    req:HttpRequest
) -> impl Responder {
    if let Some(user_id) = req.extensions().get::<Uuid>().cloned(){
        let id = id.into_inner();
        let post = query_as!(
            Post,
            r#"SELECT * FROM post WHERE id = $1 AND user_id = $2"#,
            id,
            user_id
        ).fetch_one(&data.db).await;
        match post {
            Ok(post) => {
                let update_post = query_as!(
                    Post,
                    r#"UPDATE post SET title=$1,content=$2 RETURNING *"#,
                    body.title.clone().unwrap_or_else(||post.title),
                    body.content.clone().unwrap_or_else(||post.content),
                ).fetch_one(&data.db).await;

                match update_post {
                    Ok(_)=>{
                        let response =  json!({
                            "message":"update success",
                            "status":"success",
                            "data":body
                        });
                        HttpResponse::Ok().json(response)
                    },
                    Err(err)=>{
                        HttpResponse::InternalServerError().json(
                            serde_json::json!({"status": "error", "message": format!("{}",err)}),
                        )
                    }
                }
            },
            Err(err) => {
                if err.to_string().contains("no rows returned by a query that expected to return at least one row") {
                    return HttpResponse::NotFound().json(
                        json!({"status":"failed","messsage":"data not found"})
                    )
                }
                else {
                HttpResponse::InternalServerError().json(
                    serde_json::json!({"status": "error", "message": "Something bad happened when fetching all posts"}),
                )
                }
            },
        }
    
    }else{
        return HttpResponse::Unauthorized().json(json!({"message":"invalid user id","status":"failed"}))
    }
}



pub fn public_post_config(conf: &mut web::ServiceConfig) {
    let public_scope = web::scope("/post")
    .service(get_all_post)
    .service(get_one_post)
.wrap(Authentication) 
.service(create_post_handlers)
.service(delete_post_by_id)
.service(update_post_by_id);

    conf.service(public_scope);
}
