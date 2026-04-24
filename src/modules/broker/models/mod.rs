use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::modules::broker::errors::PublisherErrors;

pub type BrokerResult<T> = Result<T, PublisherErrors>;

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq)]
#[getset(get = "pub")]
pub struct PublishMessage {
    task_id: Uuid,
    user_id: String,
    task_type: TaskType,
    payload: Value,
}

impl PublishMessage {
    pub fn new(task_id: Uuid, user_id: String, task_type: TaskType, payload: Value) -> Self {
        Self {
            task_id,
            user_id,
            task_type,
            payload,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ToSchema)]
pub enum TaskType {
    #[serde(rename = "images.generate")]
    ImageGenerate,
    #[serde(rename = "images.edit")]
    ImageEdit,
    #[serde(rename = "videos.generate")]
    VideosGenerate,
    #[serde(rename = "videos.animate")]
    VideosAnimate,
}

impl ToString for TaskType {
    fn to_string(&self) -> String {
        match self {
            Self::ImageGenerate => "images.generate".into(),
            Self::ImageEdit => "images.edit".into(),
            Self::VideosGenerate => "videos.generate".into(),
            Self::VideosAnimate => "videos.animate".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ServiceExchange {
    #[serde(rename = "images.tasks")]
    ImagesExchange,
    #[serde(rename = "videos.tasks")]
    VideosExchange,
}

impl TaskType {
    pub fn exchange(&self) -> ServiceExchange {
        match &self {
            Self::ImageGenerate | Self::ImageEdit => ServiceExchange::ImagesExchange,
            Self::VideosAnimate | Self::VideosGenerate => ServiceExchange::VideosExchange,
        }
    }
}

impl ServiceExchange {
    pub fn to_service_name(&self) -> String {
        match self {
            Self::ImagesExchange => "image-generation".to_owned(),
            Self::VideosExchange => "video-generation".to_owned(),
        }
    }
}

impl ToString for ServiceExchange {
    fn to_string(&self) -> String {
        match self {
            Self::ImagesExchange => "images.tasks".to_string(),
            Self::VideosExchange => "videos.tasks".to_string(),
        }
    }
}
