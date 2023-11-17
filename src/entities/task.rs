#[cfg(test)]
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug)]
#[cfg_attr(test, derive(Deserialize))]
pub struct Task {
    pub id: Uuid,
    pub description: String,
}
