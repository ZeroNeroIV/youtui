use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub channel: Option<String>,
    pub duration: Option<String>,
    pub thumbnail: Option<String>,
    pub description: Option<String>,
    pub views: Option<i64>,
    pub upload_date: Option<String>,
}
