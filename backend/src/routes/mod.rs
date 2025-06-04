#![allow(dead_code)]
#![allow(unused_imports)]

pub mod admin;
pub mod auth;
pub mod links;
pub mod users;
pub mod root;
pub mod tags;
pub mod categories;
pub mod profile;

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
    update_link_handler,
    increment_click_count_handler,
    get_link_tags_handler,
    add_link_tag_handler,
    remove_link_tag_handler,
};

pub use users::{
    create_user_handler,
};

pub use tags::{
    get_tags_handler,
    create_tag_handler,
    delete_tag_handler,
};

pub use categories::{
    get_categories_handler,
    create_category_handler,
    delete_category_handler,
};

pub use profile::{
    get_profile_handler,
    update_profile_handler,
    change_password_handler,
};

pub use root::root_handler;