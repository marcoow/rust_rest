use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
}
