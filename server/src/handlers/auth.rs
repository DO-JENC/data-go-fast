use argon2::{
  Argon2,
  password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng as Osrng,
  },
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::{
  TypedHeader,
  extract::CookieJar,
  headers::{Authorization, authorization::Bearer},
};
use chrono::{Duration as ChronoDuration, Utc};
use cookie::{Cookie, SameSite};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::{Rng, rng};
use redis::AsyncCommands;
use std::env;
use time::Duration as TimeDuration;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::{
  AppState,
  errors::AppError,
  infra::database::{
    auth::{get_refresh_token, revoke_refresh_token, save_refresh_token},
    user::{create_user, find_user_by_email, find_user_by_id},
  },
  models::auth::{AuthBody, AuthPayload, Claims},
};

#[instrument(skip_all)]
pub async fn logout(
  State(state): State<AppState>,
  jar: CookieJar,
  bearer: Option<TypedHeader<Authorization<Bearer>>>,
) -> Result<impl IntoResponse, AppError> {
  let refresh_token = jar
    .get("refresh_token")
    .map(|c| c.value().to_string())
    .ok_or_else(|| {
      warn!("Logout attempted without refresh token cookie");
      AppError::Unauthorized("No refresh token found")
    })?;

  let mut redis_conn = state.redis_connection.clone();

  // Block list the access token if one was provided
  if let Some(TypedHeader(auth)) = bearer {
    // Decode without validating expiry so we can still blocklist tokens that are about to expire
    let mut validation = Validation::default();
    validation.validate_exp = false;

    let token_data = decode::<Claims>(
      auth.token(),
      &DecodingKey::from_secret(state.jwt_secret.as_ref()),
      &validation,
    )
    .map_err(|e| {
      error!("Failed to decode access token for logout: {:?}", e);
      AppError::Unauthorized("Invalid access token")
    })?;

    let now = Utc::now().timestamp() as usize;
    let remaining = token_data.claims.exp.saturating_sub(now);

    if remaining > 0 {
      let blocklist_key = format!("blocklist:{}", auth.token());
      let _: () = redis_conn
        .set_ex(&blocklist_key, "1", remaining as u64)
        .await
        .map_err(|e| {
          error!("Failed to blocklist token in Redis: {:?}", e);
          AppError::Internal("Redis error")
        })?;
      info!(user_id = %token_data.claims.sub, "Access token blocklisted");
    }
  }

  let redis_key = format!("refresh_token:{}", refresh_token);

  // Remove from Redis
  let _: () = redis_conn.del(&redis_key).await.map_err(|e| {
    error!("Failed to remove refresh token from Redis: {:?}", e);
    AppError::Internal("Redis error")
  })?;

  // Remove from Postgres
  revoke_refresh_token(&state.pool, &refresh_token)
    .await
    .map_err(|e| {
      error!("Failed to revoke refresh token from database: {:?}", e);
      AppError::Internal("Database error")
    })?;

  info!("User logged out successfully");
  // Clear cookie
  Ok((jar.remove(Cookie::from("refresh_token")), StatusCode::OK))
}

#[instrument(skip_all, fields(user_id = %user_id))]
async fn generate_refresh_token(state: AppState, user_id: Uuid) -> Result<String, AppError> {
  // Generate a new refresh token,
  let mut bytes = [0u8; 32];
  rng().fill_bytes(&mut bytes);
  let refresh_token: String = hex::encode(bytes);

  // Save to Postgres
  let expires_at = save_refresh_token(&state.pool, user_id, &refresh_token, 7)
    .await
    .map_err(|e| {
      error!("Failed to save refresh token to database: {:?}", e);
      AppError::Internal("Database error")
    })?;

  // Save to Redis (using TTL from days)
  let mut redis_conn = state.redis_connection.clone();
  let redis_key = format!("refresh_token:{}", refresh_token);
  let ttl = (expires_at - Utc::now()).num_seconds() as u64;

  let _: () = redis_conn
    .set_ex(&redis_key, user_id.to_string(), ttl)
    .await
    .map_err(|e| {
      error!("Failed to save refresh token to Redis: {:?}", e);
      AppError::Internal("Redis error")
    })?;

  Ok(refresh_token)
}

#[instrument(skip_all)]
pub async fn refresh_token(
  State(state): State<AppState>,
  jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
  let refresh_token: String = jar
    .get("refresh_token")
    .map(|c| c.value().to_string())
    .ok_or_else(|| {
      warn!("Token refresh attempted without cookie");
      AppError::Unauthorized("No refresh token found")
    })?;

  let mut redis_conn = state.redis_connection.clone();
  let redis_key = format!("refresh_token:{}", refresh_token);

  // Try Redis first
  let user_id: Uuid = match redis_conn.get::<_, Option<String>>(&redis_key).await {
    Ok(Some(id_str)) => Uuid::parse_str(&id_str).map_err(|e| {
      error!("Invalid ID format in Redis: {:?}", e);
      AppError::Internal("Invalid ID format in cache")
    })?,
    _ => {
      // Fallback to Postgres
      let token_row = get_refresh_token(&state.pool, &refresh_token)
        .await
        .map_err(|e| {
          error!("Failed to fetch refresh token from database: {:?}", e);
          AppError::Internal("Database error")
        })?
        .ok_or_else(|| {
          warn!("Invalid refresh token provided");
          AppError::Unauthorized("Invalid refresh token")
        })?;

      if token_row.expires_at < Utc::now().naive_utc() {
        warn!(user_id = %token_row.user_id, "Refresh token expired");
        return Err(AppError::Unauthorized("Refresh token expired"));
      }

      let remaining_ttl = (token_row.expires_at - Utc::now().naive_utc()).num_seconds();
      if remaining_ttl <= 0 {
        return Err(AppError::Unauthorized("Refresh token expired"));
      }

      // Sync back to Redis for faster subsequent access
      let _: () = redis_conn
        .set_ex(
          &redis_key,
          token_row.user_id.to_string(),
          remaining_ttl as u64,
        )
        .await
        .map_err(|e| {
          error!("Failed to sync refresh token to Redis: {:?}", e);
          AppError::Internal("Redis error")
        })?;

      token_row.user_id
    }
  };

  let user = find_user_by_id(&state.pool, user_id)
    .await
    .map_err(|e| {
      error!("Failed to fetch user {} from database: {:?}", user_id, e);
      AppError::Internal("Database error")
    })?
    .ok_or_else(|| {
      error!("User {} not found for valid refresh token", user_id);
      AppError::Unauthorized("User not found")
    })?;

  let access_token = generate_jwt(user.id, user.email)?;
  let new_cookie = build_cookie(refresh_token);

  info!(user_id = %user.id, "Token refreshed successfully");

  Ok((
    jar.add(new_cookie),
    Json(AuthBody {
      access_token,
      token_type: "Bearer".to_string(),
    }),
  ))
}

fn validate_password(password: &str) -> Result<&str, AppError> {
  if password.len() < 8 {
    return Err(AppError::Unprocessable(
      "Password must be at least 8 characters long",
    ));
  }

  let has_special = password.chars().any(|c| !c.is_alphanumeric());

  if !has_special {
    return Err(AppError::Unprocessable(
      "Password must contain at least one special character",
    ));
  }
  Ok(password)
}

#[instrument(skip_all, fields(email = %payload.email))]
pub async fn signup(
  State(state): State<AppState>,
  jar: CookieJar,
  Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, AppError> {
  info!("Processing signup request");
  let salt = SaltString::generate(&mut Osrng);
  let argon2 = Argon2::default();

  let _ = validate_password(&payload.password)?.as_bytes();
  let password_hash = argon2
    .hash_password(payload.password.as_bytes(), &salt)
    .map_err(|e| {
      error!("Password hashing failed: {:?}", e);
      AppError::Internal("Error hashing password")
    })?
    .to_string();

  let user = create_user(&state.pool, &payload.email, &password_hash)
    .await
    .map_err(|e| {
      if let Some(db_err) = e.as_database_error()
        && db_err.is_unique_violation()
      {
        warn!("Signup attempted with existing email: {}", payload.email);
        return AppError::Conflict("User already exists");
      }
      error!("User creation failed for {}: {:?}", payload.email, e);
      AppError::Internal("Internal server error")
    })?;

  let access_token = generate_jwt(user.id, user.email.clone())?;
  let refresh_token: String = generate_refresh_token(state, user.id).await?;
  let cookie = build_cookie(refresh_token);

  info!(user_id = %user.id, "User signed up successfully");

  Ok((
    jar.add(cookie),
    (
      StatusCode::CREATED,
      Json(AuthBody {
        access_token,
        token_type: "Bearer".to_string(),
      }),
    ),
  ))
}

#[instrument(skip_all, fields(email = %payload.email))]
pub async fn login(
  State(state): State<AppState>,
  jar: CookieJar,
  Json(payload): Json<AuthPayload>,
) -> Result<impl IntoResponse, AppError> {
  info!("Processing login request");
  let user = find_user_by_email(&state.pool, &payload.email)
    .await
    .map_err(|e| {
      error!("Failed to fetch user by email: {:?}", e);
      AppError::Internal("Internal server error")
    })?
    .ok_or_else(|| {
      warn!("Login failed: user not found");
      AppError::Unauthorized("Invalid credentials")
    })?;

  let parsed_hash = PasswordHash::new(&user.hash_password).map_err(|e| {
    error!("Invalid password hash format for user {}: {:?}", user.id, e);
    AppError::Internal("Invalid password hash format in database")
  })?;

  let is_valid = Argon2::default()
    .verify_password(payload.password.as_bytes(), &parsed_hash)
    .is_ok();

  if !is_valid {
    warn!(user_id = %user.id, "Login failed: invalid password");
    return Err(AppError::Unauthorized("Invalid credentials"));
  }

  let access_token = generate_jwt(user.id, user.email.clone())?;
  let refresh_token: String = generate_refresh_token(state, user.id).await?;
  let cookie = build_cookie(refresh_token);

  info!(user_id = %user.id, "User logged in successfully");

  Ok((
    jar.add(cookie),
    Json(AuthBody {
      access_token,
      token_type: "Bearer".to_string(),
    }),
  ))
}

fn build_cookie(refresh_token: String) -> Cookie<'static> {
  let mut cookie = Cookie::build(("refresh_token", refresh_token))
    .path("/")
    .http_only(true)
    .same_site(SameSite::Lax)
    .max_age(TimeDuration::days(7));

  if env::var("APP_ENV").unwrap_or_default() == "production" {
    cookie = cookie.secure(true);
  }
  cookie.build().into_owned()
}

fn generate_jwt(user_id: Uuid, email: String) -> Result<String, AppError> {
  let secret = env::var("JWT_SECRET").map_err(|e| {
    error!("JWT_SECRET environment variable missing: {:?}", e);
    AppError::Internal("JWT_SECRET not set")
  })?;

  let exp = Utc::now()
    .checked_add_signed(ChronoDuration::hours(24))
    .expect("valid timestamp")
    .timestamp() as usize;

  let claims = Claims {
    sub: user_id,
    email,
    exp,
  };

  encode(
    &Header::default(),
    &claims,
    &EncodingKey::from_secret(secret.as_ref()),
  )
  .map_err(|e| {
    error!("JWT encoding failed for user {}: {:?}", user_id, e);
    AppError::Internal("Error generating token")
  })
}
