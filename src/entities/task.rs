#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[cfg_attr(test, derive(Deserialize))]
pub struct Task {
    pub id: i32,
    pub description: String,
}
