use chrono::DateTime;
use chrono::offset::Utc;
use diesel::prelude::*;
use diesel;
use hex;
use rand::{self, Rng};
use rocket::http::{Cookie, Cookies};

use user::{AuthUser, User};
use db::Db;
use db::schema::{sessions, users};
use config;

pub mod routes;
mod html;


/// A login-provider. Is able to authenticate a user.
pub trait Provider {
    fn auth(&self, username: &str, secret: &str, db: &Db) -> Result<User, LoginError>;
}



pub fn login(
    username: &str,
    secret: &str,
    provider: &Provider,
    cookies: Cookies,
    db: &Db,
) -> Result<AuthUser, LoginError> {
    // Find the user with the given username.
    let user = match User::from_username(username, db) {
        Some(u) => u,
        None => return Err(LoginError::UserNotFound),
    };

    // Try to authenticate with the given provider. If it fails, we return an
    // error.
    provider.auth(username, secret, db)?;

    // Create a session in the database and set it as cookie.
    let session = Session::create_for(&user, cookies, db);

    Ok(AuthUser::new(user, session))
}


#[derive(Debug)]
pub enum LoginError {
    /// There is not user with the given username.
    UserNotFound,

    /// A user was found, but the given password/secret is not correct.
    SecretIncorrect,

    /// A user was found, but cannot be authenticated with this provider.
    ProviderNotUsable,
}




#[derive(Debug, Clone, Eq, PartialEq, Identifiable, Queryable, Associations)]
// #[belongs_to(User)]
pub struct Session {
    pub id: Vec<u8>,
    pub user_id: i64,
    pub birth: DateTime<Utc>,
}

impl Session {
    pub fn create_for(user: &User, mut cookies: Cookies, db: &Db) -> Self {
        // Generate a random session id.
        let mut id = [0u8; config::SESSION_ID_LEN];
        let mut rng = rand::os::OsRng::new()
            .expect("could not use system rng");
        rng.fill_bytes(&mut id);

        // Insert session id linked with the user id into the database.
        #[derive(Debug, Clone, Eq, PartialEq, Insertable)]
        #[table_name = "sessions"]
        pub struct NewSession {
            pub id: Vec<u8>,
            pub user_id: i64,
        }

        let new_session = NewSession {
            id: id.to_vec(),
            user_id: user.id(),
        };
        let inserted_session = diesel::insert(&new_session)
            .into(sessions::table)
            .get_result::<Session>(&*db.conn())
            .unwrap();

        // Encode session id as hex and set it as cookie.
        let encoded = hex::encode(&id);
        cookies.add(Cookie::new(config::SESSION_COOKIE_NAME, encoded));

        inserted_session
    }

    pub fn verify(cookies: Cookies, db: &Db) -> Option<AuthUser> {
        // TODO: once associations work again, use a join here instead of two
        // queries.

        cookies.get(config::SESSION_COOKIE_NAME)
            .and_then(|cookie| hex::decode(cookie.value()).ok())
            .and_then(|session_id| {

                if session_id.len() != config::SESSION_ID_LEN {
                    return None;
                }

                // Try to find session id and the associated user.
                sessions::table
                    .find(session_id)
                    .first::<Session>(&*db.conn())
                    .optional()
                    .unwrap()
            })
            .and_then(|session| {
                users::table
                    .find(session.user_id)
                    .first::<User>(&*db.conn())
                    .optional()
                    .unwrap()
                    .map(|user| AuthUser::new(user, session))
            })
    }

    /// Ends a login session, removing the entry from the database and removing
    /// the cookie.
    ///
    /// This function assumes the user was authenticated via session cookie.
    pub fn destroy(self, mut cookies: Cookies, db: &Db) {
        // Since we assume the user was authenticated via session id, we know
        // the cookie jar contains such a cookie and the cookie is a valid
        // hex string.
        let session_id = hex::decode(
            cookies.get(config::SESSION_COOKIE_NAME).unwrap().value()
        ).unwrap();

        // Remove from database.
        diesel::delete(sessions::table.find(session_id))
            .execute(&*db.conn())
            .expect("failed to delete session entry from database");

        // Remove from cookie jar.
        cookies.remove(Cookie::named(config::SESSION_COOKIE_NAME));
    }
}
