use actix_web::{post, web, Error, HttpResponse, Responder};
use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHasher, PasswordVerifier};
use serde_json::json;
use sqlx::query_as;
use validator::Validate;
use crate::AppState;
use crate::utils::jwt::TokenClaims;
use super::auth_models::{Register,Login,User,UserPayload};

#[post("/register")]
pub async fn register(
    body: web::Json<Register>,
    db_conn: web::Data<AppState>,
) -> impl Responder {
    // Generate salt and hash password
    let salt: SaltString = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = match argon2.hash_password(body.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("Error hashing password: {}", e)
            }))
        }
    };

    let mut user_input = body.into_inner();

    // Validate input
    if let Err(errors) = user_input.validate() {
        return HttpResponse::BadRequest().json(json!({
            "status": "fail",
            "message": errors
        }));
    }

    user_input.password = password_hash;

    // Insert user into database
    let new_user = query_as!(
        Register,
        r#"INSERT INTO "user" (email, password) VALUES ($1, $2) RETURNING *"#,
        user_input.email,
        user_input.password
    )
    .fetch_one(&db_conn.db)
    .await;
    match new_user {
        Ok(user) => {
            let payload:UserPayload = UserPayload {
                id: user.id.expect("invalid uuid"),
                email: user.email,
            };

            HttpResponse::Created().json(json!({
                "status": "success",
                "data":payload
            }))
        },            
        Err(err) => {
            if err.to_string().contains("duplicate key value violates unique constraint") {
                HttpResponse::BadRequest().json(json!({
                    "status": "fail",
                    "message": "User with that email or username already exists"
                }))
            } else {
                HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("{:?}", err)
                }))
            }
        }
    }
}

#[post("/login")]
pub async fn login(
    body:web::Json<Login>,
    db_conn:web::Data<AppState>
) -> impl Responder {
    let user_result = query_as!(
        User,
        r#"SELECT * FROM "user" WHERE email = $1"#,
        body.email
    )
    .fetch_one(&db_conn.db)
    .await;

    match user_result{
        Ok(user)=>{
            let password_hash = user.password;
            let argon2 = Argon2::default();
            let parsed_hash = argon2::PasswordHash::new(&password_hash).unwrap();
            let result = argon2.verify_password(body.password.as_bytes(), &parsed_hash);

            match result {
            Ok(_)=>{
                let user_payload :UserPayload = UserPayload{
                    id:user.id,
                    email:user.email
                };
                let token:String= TokenClaims::generate_token(user_payload).unwrap();
                Ok::<HttpResponse, Error>(HttpResponse::Ok().json(json!({"status":"success","token":token,"message":"login success"})))
            },
            Err(_err) => Ok(HttpResponse::Unauthorized().json(json!({"message":"error when login"})))
            }
        },
        Err(_err)=>{
            Ok(HttpResponse::Unauthorized().json(json!({"message":"error when login"})))
        }
    }
}

pub fn auth_config(config:&mut web::ServiceConfig){
    config.service(
        web::scope("/auth")
        .service(register)
        .service(login)
    );
}