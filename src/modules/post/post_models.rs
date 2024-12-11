use sqlx::types::chrono::NaiveTime;
use serde::{Serialize,Deserialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub user_id: Option<Uuid>,
    pub create_at: Option<NaiveTime>,
    pub updated_at: Option<NaiveTime>,
}

#[derive(Serialize,Deserialize,Validate)]
pub struct NewPost{
    #[validate(length(min="5",message="please add your title"))]
    pub title:String,
    #[validate(length(min="20",message="please add your content"))]
    pub content:String,
    pub user_id:Option<Uuid>
}

#[derive(Serialize,Deserialize)]
pub struct UpdatePost{
    pub title:Option<String>,
    pub content:Option<String>
}