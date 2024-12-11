use jsonwebtoken::{encode, EncodingKey, Header, TokenData,decode,DecodingKey,Validation ,errors::Error as JwtError};

use crate::modules::auth::auth_models::UserPayload;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TokenClaims {
    pub iat : i64,
    pub exp : i64,
    pub user :UserPayload,
}

#[derive(Deserialize,Serialize)]
pub struct JwtUserToken{
        pub user: UserPayload,
        pub iat: i64,    
        pub exp: i64
}

impl TokenClaims {
    pub fn generate_token(data:UserPayload) -> Result<String, String> {
        let max_age:i64 = 60 * 60 * 24;
        let iat = chrono::Utc::now().timestamp();
        let exp = iat + max_age;
        let token = TokenClaims {
            iat,
            exp,
            user: data,
        };

        let jwt_secret:EncodingKey = EncodingKey::from_secret("secret_key".as_bytes());
        let token = encode( &Header::default(),&token, &jwt_secret).unwrap();
        Ok(token)
    }
}

pub fn decode_token(token: String) -> Result<TokenData<JwtUserToken>, String> {
    let user = decode::<JwtUserToken>(
        &token,
        &DecodingKey::from_secret("secret_key".as_bytes()),
        &Validation::default(),
    ).map_err(|e: JwtError| e.to_string());
    user
}