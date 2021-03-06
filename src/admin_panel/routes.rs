use std::collections::HashMap;

use diesel::prelude::*;
use rocket::config::Config;
use rocket::State;

use db::Db;
use dict::{self, Locale};
use errors::*;
use state::CurrentAppState;
use super::html;
use template::Page;
use user::{AuthAdmin, Role};


#[get("/admin_panel")]
pub fn index(
    _admin: AuthAdmin,
    locale: Locale,
    db: State<Db>,
    config: State<Config>,
) -> Result<Page> {
    use diesel::expression::dsl::*;
    use db::schema::users;

    // Calculate some stats.
    let counts = users::table
        .group_by(users::role)
        // We have to use raw sql here, because diesel is not powerful enough
        // to express this yet. See diesel-rs/diesel#772
        .select(sql("role, count(*)"))
        .load::<(Role, i64)>(&*db.conn()?)?
        .into_iter()
        .collect::<HashMap<_, _>>();

    let stats = html::Stats {
        num_admins: counts.get(&Role::Admin).cloned().unwrap_or(0) as u64,
        num_tutors: counts.get(&Role::Tutor).cloned().unwrap_or(0) as u64,
        num_students: counts.get(&Role::Student).cloned().unwrap_or(0) as u64,
    };

    Page::empty()
        .with_title("Admin Panel")
        .with_content(html::index(locale, &stats, &config))
        .with_active_nav_route("/admin_panel")
        .make_ok()
}


#[get("/admin_panel/state")]
pub fn state(
    _admin: AuthAdmin,
    locale: Locale,
    db: State<Db>,
) -> Result<Page> {
    let app_state = CurrentAppState::load(&db)?;

    Page::empty()
        .with_title(dict::new(locale).admin_panel.state_title())
        .with_active_nav_route("/admin_panel")
        .with_content(html::state(locale, &app_state))
        .make_ok()
}
