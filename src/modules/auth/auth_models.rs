use serde::{Deserialize,Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize,Serialize,Validate)]
pub struct Register {
    pub id:Option<Uuid>,
    #[validate(email(message="email must be valid"))]
    pub email:String,
    #[validate(length(min="8",message="please add your password"))]
    pub password:String
}

#[derive(Deserialize,Serialize)]
pub struct User {
    pub id:Uuid,
    pub email:String,
    pub password:String
}

#[derive(Deserialize,Serialize)]
pub struct Login {
    pub email:String,
    pub password:String
}

#[derive(Debug,Deserialize,Serialize)]
pub struct UserPayload {
    pub id: Uuid,
    pub email: String,
}