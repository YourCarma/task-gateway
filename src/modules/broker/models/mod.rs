use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;


use crate::modules::broker::errors::PublisherErrors;

pub type BrokerResult<T> = Result<T, PublisherErrors>;


#[derive(Serialize, Deserialize,Getters, Debug, Clone, PartialEq)]
#[getset(get = "pub")]
pub struct PublishMessage{
    user_id: String,
    task_type: TaskType,
    payload: Value
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TaskType{
    #[serde(rename="images.generate")]
    ImageGenerate,
    #[serde(rename="images.edit")]
    ImageEdit,
    #[serde(rename="videos.generate")]
    VideosGenerate,
    #[serde(rename="videos.animate")]
    VideosAnimate
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ServiceType{
    #[serde(rename="images.tasks")]
    ImagesExchange,
    #[serde(rename="videos.tasks")]
    VideosExchange
}


impl TaskType {
    pub fn exchange(&self) -> ServiceType{
        match &self {
            Self::ImageGenerate | Self::ImageEdit => ServiceType::ImagesExchange,
            Self::VideosAnimate | Self::VideosGenerate => ServiceType::VideosExchange
        }
    }
}

impl ServiceType {
    pub fn to_service_name(&self) -> String{
        match self {
            Self::ImagesExchange => "image-generation".to_owned(),
            Self::VideosExchange => "video-generation".to_owned()
        }
    }
}