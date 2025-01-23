// src/database.rs

use mysql::*;
use mysql::prelude::*;
use std::env;
use dotenv::dotenv;
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
}

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

pub struct Database {
    pool: Pool,
}

impl Database {
    pub fn create_user(&self, req: CreateUserRequest) -> Result<User, AuthError> {
        let mut conn = self.pool.get_conn()?;
        
        // Check if username exists
        let exists: Option<i32> = conn
            .exec_first(
                "SELECT id FROM users WHERE username = :username",
                params! {
                    "username" => &req.username,
                }
            )?;
            
        if exists.is_some() {
            return Err(AuthError::UsernameTaken);
        }
        
        // Hash password
        let password_hash = hash(req.password.as_bytes(), DEFAULT_COST)
            .map_err(|_| AuthError::HashingError)?;
            
        // Insert user with optional fields
        let query = "INSERT INTO users 
            (username, password_hash, email, full_name) 
            VALUES (:username, :password_hash, :email, :full_name)";
        
        conn.exec_drop(
            query,
            params! {
                "username" => &req.username,
                "password_hash" => &password_hash,
                "email" => &req.email,
                "full_name" => &req.full_name,
            }
        )?;
        
        let id = conn.last_insert_id() as i32;
        
        Ok(User {
            id,
            username: req.username,
        })
    }

    pub fn get_all_users(&self) -> Result<Vec<User>, AuthError> {
        let mut conn = self.pool.get_conn()?;
        
        let users: Vec<User> = conn
            .query("SELECT id, username FROM users ORDER BY created_at DESC")?
            .into_iter()
            .map(|row| {
                let (id, username): (i32, String) = mysql::from_row(row);
                User { id, username }
            })
            .collect();
        
        Ok(users)
    }

    pub fn new() -> Result<Self> {
        dotenv().ok();
        
        let url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
            
        let pool = Pool::new(url.as_str())?;
        
        Ok(Database { pool })
    }
    
    pub fn init(&self) -> Result<()> {
        let mut conn = self.pool.get_conn()?;
        
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS users (
                id INT PRIMARY KEY AUTO_INCREMENT,
                username VARCHAR(255) UNIQUE NOT NULL,
                password_hash VARCHAR(255) NOT NULL,
                email VARCHAR(255),
                full_name VARCHAR(255),
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        )?;
        
        Ok(())
    }

    pub fn register_user(&self, req: RegisterRequest) -> Result<User, AuthError> {
        let mut conn = self.pool.get_conn()?;
        
        // Check if username exists
        let exists: Option<i32> = conn
            .exec_first(
                "SELECT id FROM users WHERE username = :username",
                params! {
                    "username" => &req.username,
                }
            )?;
            
        if exists.is_some() {
            return Err(AuthError::UsernameTaken);
        }
        
        // Hash password
        let password_hash = hash(req.password.as_bytes(), DEFAULT_COST)
            .map_err(|_| AuthError::HashingError)?;
            
        // Insert user
        conn.exec_drop(
            "INSERT INTO users (username, password_hash) VALUES (:username, :password_hash)",
            params! {
                "username" => &req.username,
                "password_hash" => &password_hash,
            }
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
            .exec_first(
                "SELECT id, username, password_hash FROM users WHERE username = :username",
                params! {
                    "username" => &req.username,
                }
            )?
            .ok_or(AuthError::InvalidCredentials)?;
            
        if !verify(req.password.as_bytes(), &password_hash)
            .map_err(|_| AuthError::HashingError)? {
            return Err(AuthError::InvalidCredentials);
        }
        
        Ok(User { id, username })
    }
}