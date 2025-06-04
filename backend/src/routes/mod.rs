#![allow(dead_code)]
#![allow(unused_imports)]

pub mod admin;
pub mod auth;
pub mod links;
pub mod users;
pub mod root;

// Re-export all route handlers
pub use admin::{
    get_user_stats_handler,
    get_all_links_handler,
    delete_user_handler,
    delete_any_link_handler,
};

pub use auth::{
    login_handler,
    register_handler,
    logout_handler,
    verify_email_handler,
};

pub use links::{
    create_link_handler,
    get_links_handler,
    delete_link_handler,
    increment_click_count_handler,
};

pub use users::{
    create_user_handler,
};

pub use root::root_handler;