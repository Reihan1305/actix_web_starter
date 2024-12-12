use sqlx::types::chrono::NaiveDateTime;
use serde::{Serialize,Deserialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub create_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize,Deserialize,Validate)]
pub struct NewPost{
    #[validate(length(min="5",message="please add your title"))]
    pub title:String,
    #[validate(length(min="20",message="please add your content"))]
    pub content:String,
}

#[derive(Serialize,Deserialize)]
pub struct UpdatePost{
    pub title:Option<String>,
    pub content:Option<String>
}