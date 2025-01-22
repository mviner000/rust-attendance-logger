// src/auth.rs

use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::database::Database;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Database(#[from] mysql::Error),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Username already exists")]
    UsernameTaken,
    #[error("Password hashing failed")]
    HashingError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl Database {
    pub fn register_user(&self, req: RegisterRequest) -> Result<User, AuthError> {
        let mut conn = self.pool.get_conn()?;
        
        // Check if username exists
        let exists: Option<i32> = conn
            .query_first("SELECT id FROM users WHERE username = ?", (&req.username,))?;
            
        if exists.is_some() {
            return Err(AuthError::UsernameTaken);
        }
        
        // Hash password
        let password_hash = hash(req.password.as_bytes(), DEFAULT_COST)
            .map_err(|_| AuthError::HashingError)?;
            
        // Insert user
        conn.exec_drop(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            (&req.username, &password_hash),
        )?;
        
        let id = conn.last_insert_id() as i32;
        
        Ok(User {
            id,
            username: req.username,
        })
    }
    
    pub fn login_user(&self, req: LoginRequest) -> Result<User, AuthError> {
        let mut conn = self.pool.get_conn()?;
        
        let (id, username, password_hash): (i32, String, String) = conn
            .query_first("SELECT id, username, password_hash FROM users WHERE username = ?",(req.username,))?
            .ok_or(AuthError::InvalidCredentials)?;
            
        if !verify(req.password.as_bytes(), &password_hash)
            .map_err(|_| AuthError::HashingError)? {
            return Err(AuthError::InvalidCredentials);
        }
        
        Ok(User { id, username })
    }
}