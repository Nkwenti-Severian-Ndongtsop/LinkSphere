mod links;
mod users;
mod root;
pub mod auth;
pub mod admin;

pub use links::{get_links_handler, delete_link_handler, increment_click_count_handler, create_link_handler};
pub use users::create_user_handler;
pub use root::root_handler;
pub use admin::{
    get_all_users_handler,
    get_all_links_handler,
    delete_user_handler,
    delete_any_link_handler,
    get_user_stats_handler,
};