pub mod user_service;
pub mod auth_service;
pub mod product_service;

use user_service::{ get_all_users };
use auth_service::{ login, register };
