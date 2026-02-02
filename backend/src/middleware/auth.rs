use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures::future::Ready;
use crate::middleware::jwt::JwtService;

/// Extractor for authenticated user
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub email: String,
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;
    
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let jwt_service = req.app_data::<web::Data<JwtService>>();
        
        if let Some(jwt) = jwt_service {
            // Get Authorization header
            let auth_header = req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());
            
            if let Some(header_value) = auth_header {
                if header_value.starts_with("Bearer ") {
                    let token = &header_value[7..];
                    
                    if let Ok(claims) = jwt.validate_token(token) {
                        if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
                            return futures::future::ready(Ok(AuthUser {
                                user_id,
                                email: claims.email,
                            }));
                        }
                    }
                }
            }
        }
        
        // Return unauthorized error
        futures::future::ready(Err(actix_web::error::ErrorUnauthorized(
            "Invalid or missing authentication token"
        )))
    }
}

/// Optional auth - returns None if no valid token
#[derive(Debug, Clone)]
pub struct OptionalAuth(pub Option<AuthUser>);

impl FromRequest for OptionalAuth {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;
    
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let jwt_service = req.app_data::<web::Data<JwtService>>();
        
        if let Some(jwt) = jwt_service {
            let auth_header = req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());
            
            if let Some(header_value) = auth_header {
                if header_value.starts_with("Bearer ") {
                    let token = &header_value[7..];
                    
                    if let Ok(claims) = jwt.validate_token(token) {
                        if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
                            return futures::future::ready(Ok(OptionalAuth(Some(AuthUser {
                                user_id,
                                email: claims.email,
                            }))));
                        }
                    }
                }
            }
        }
        
        futures::future::ready(Ok(OptionalAuth(None)))
    }
}
