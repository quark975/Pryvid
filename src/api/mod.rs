use futures::future::join_all;
use isahc::prelude::*;
use isahc::HttpClient;
use lazy_static::lazy_static;
use rand::{self, seq::SliceRandom};
use regex::Regex;
use serde::Serialize;
use serde::{self, Deserialize};
use serde_json::Value;
use std::io;
use std::sync::RwLock;
use std::sync::{Arc, PoisonError};
use std::time::Duration;
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Instance already exists")]
    InstanceExists,
    #[error("Request failed")]
    RequestError(#[from] isahc::Error),
    #[error("IO error")]
    IoError(#[from] io::Error),
    #[error("No known instances")]
    NoInstances,
    #[error("Thread with lock panicked")]
    PoisonError,
    #[error("Tried to get instance at non-existent index")]
    OutOfBounds,
    #[error("Failed to deserialize")]
    DeserializeError(#[from] serde_json::Error),
    #[error("Must have at least one instance")]
    AtLeastOneInstance,
    #[error("Instance not found")]
    InstanceNotFound,
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::PoisonError
    }
}

// Responses
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub version: String,
    pub open_registrations: bool,
}

#[derive(Debug, Deserialize)]
pub struct InstanceResponse {
    pub flag: String,
    pub region: String,
    pub stats: Option<StatsResponse>,
    #[serde(rename = "type")]
    pub protocol: String,
    pub uri: String,
}

// Utility

pub type Instances = Vec<Arc<Instance>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub uri: String,
    pub info: Arc<RwLock<InstanceInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub has_trending: bool,
    pub has_popular: bool,
    pub open_registrations: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "video")]
    Video(Video),
    #[serde(rename = "playlist")]
    Playlist(Playlist),
    #[serde(rename = "channel")]
    Channel(Channel),
}

#[derive(Debug, Deserialize)]
pub struct Video {
    pub title: String,
    #[serde(rename = "videoId")]
    pub id: String,
    #[serde(rename = "viewCount")]
    pub views: u64,
    #[serde(rename = "lengthSeconds")]
    pub length: u32,
    #[serde(rename = "videoThumbnails")]
    pub thumbnails: Vec<VideoThumbnail>,
    pub author: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
    #[serde(rename = "publishedText")]
    pub published: String,
}

#[derive(Debug, Deserialize)]
pub struct Playlist {
    pub title: String,
    #[serde(rename = "playlistId")]
    pub id: String,
    pub author: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
    #[serde(rename = "videoCount")]
    pub video_count: u64,
    pub thumbnail: String,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub title: String,
    pub id: String,
    pub verified: bool,
    pub thumbnails: Vec<AuthorThumbnail>,
    pub subscribers: u64,
}

#[derive(Debug, Deserialize)]
pub struct VideoThumbnail {
    pub quality: String,
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct AuthorThumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct InvidiousClient {
    instances: RwLock<Instances>,
    selected: RwLock<Option<Arc<Instance>>>,
    client: HttpClient,
}

// Functions
pub async fn fetch_instances() -> Result<Instances, Error> {
    let response: Vec<(String, InstanceResponse)> =
        isahc::get_async("https://api.invidious.io/instances.json?pretty=1&sort_by=type,users")
            .await?
            .json()
            .await?;

    let instances: Instances = response
        .into_iter()
        .filter_map(|(_, instance)| {
            let open_registrations = if let Some(stats) = instance.stats {
                stats.open_registrations
            } else {
                false
            };

            // TODO: Support more protocols in the future
            if instance.protocol == "https" {
                Some(Arc::new(Instance {
                    uri: instance.uri,
                    info: Arc::new(RwLock::new(InstanceInfo {
                        open_registrations,
                        has_trending: false,
                        has_popular: false,
                    })),
                }))
            } else {
                None
            }
        })
        .collect();

    // Ping in batches of 8 at a time
    for instances in instances.chunks(4).into_iter() {
        println!("{:?}", instances.iter().map(|x| x.uri.clone()).collect::<Vec<String>>());
        join_all(instances.iter().map(|x| x.update_info())).await;
    }
    // join_all(instances.iter().map(|x| x.update_info())).await;

    Ok(instances)
}

fn fix_thumbnail_urls(uri: &str, content: &mut Vec<Content>) {
    for item in content.iter_mut() {
        match item {
            Content::Video(video) => {
                for thumbnail in &mut video.thumbnails {
                    // If domain isn't present
                    if thumbnail.url.starts_with("/vi/") {
                        thumbnail.url = format!("{}{}", uri, &thumbnail.url);
                    }
                }
            }
            _ => (),
        }
    }
}

fn format_uri(uri: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^([a-z0-9]+):\/\/").unwrap();
    }
    let uri = uri.trim().trim_end_matches('/');
    if RE.is_match_at(uri, 0) {
        uri.into()
    } else {
        format!("https://{uri}")
    }
}

impl Instance {
    pub fn from_uri(uri: &str) -> Result<Instance, Error> {
        let uri = format_uri(uri);
        let response: StatsResponse = isahc::get(format!("{}/api/v1/stats", &uri))?.json()?;
        let instance = Instance {
            uri,
            info: Arc::new(RwLock::new(InstanceInfo {
                has_trending: false,
                has_popular: false,
                open_registrations: response.open_registrations,
            })),
        };
        instance.update_info();
        Ok(instance)
    }

    pub async fn update_info(&self) -> Result<(), Error> {
        let mut response = isahc::send_async(
            isahc::Request::builder()
                .uri(format!("{}/api/v1/popular", self.uri))
                .redirect_policy(isahc::config::RedirectPolicy::Limit(2))
                .timeout(Duration::from_secs(10))
                .body(())
                .unwrap(),
        )
        .await?;
        let has_popular = response.json::<Vec<Value>>().await.is_ok();

        let mut response = isahc::send_async(
            isahc::Request::builder()
                .uri(format!("{}/api/v1/trending", self.uri))
                .redirect_policy(isahc::config::RedirectPolicy::Limit(2))
                .timeout(Duration::from_secs(10))
                .body(())
                .unwrap(),
        )
        .await?;
        let has_trending = response.json::<Vec<Value>>().await.is_ok();

        let mut info = self.info.write()?;
        info.has_popular = has_popular;
        info.has_trending = has_trending;

        Ok(())
    }

    pub async fn ping(&self, endpoint: Option<&str>) -> Result<u128, Error> {
        let elapsed = Instant::now();
        let response = isahc::get_async(format!("{}{}", self.uri, endpoint.unwrap_or("/"))).await?;
        let elapsed = elapsed.elapsed();
        Ok(elapsed.as_millis())
    }
}

impl InvidiousClient {
    pub fn new(instances: Instances) -> Self {
        let client = HttpClient::new().unwrap();
        // TODO: Handle a 'no instances available' a little better
        let instances = if instances.len() == 0 {
            vec![Arc::new(Instance {
                uri: "https://vid.puffyan.us".into(),
                info: Arc::new(RwLock::new(InstanceInfo {
                    has_trending: true,
                    has_popular: true,
                    open_registrations: true,
                })),
            })]
        } else {
            instances
        };

        InvidiousClient {
            selected: RwLock::new(None),
            instances: RwLock::new(instances),
            client,
        }
    }

    // Instances
    pub fn instances(&self) -> Instances {
        self.instances.read().unwrap().clone()
    }
    pub fn selected_instance(&self) -> Option<Arc<Instance>> {
        self.selected.read().unwrap().clone()
    }
    pub fn is_selected(&self, instance: &Arc<Instance>) -> bool {
        let selected = self.selected.read().unwrap();
        if let Some(ref selected_instance) = *selected {
            return Arc::ptr_eq(selected_instance, instance);
        }
        return false;
    }
    pub fn is_added(&self, instance: &Arc<Instance>) -> bool {
        let instances = self.instances();
        instances
            .iter()
            .position(|x| x.uri == instance.uri)
            .is_some()
    }
    pub fn get_instance(&self) -> Result<Arc<Instance>, Error> {
        if let Some(ref instance) = *self.selected.read().unwrap() {
            return Ok(instance.clone());
        }

        Ok(self
            .instances()
            .choose(&mut rand::thread_rng())
            .ok_or(Error::NoInstances)?
            .clone())
    }

    pub fn push_instance(&self, instance: Arc<Instance>) -> Result<(), Error> {
        if self
            .instances()
            .iter()
            .position(|x| x.uri == instance.uri)
            .is_some()
        {
            Err(Error::InstanceExists)
        } else {
            self.instances.write().unwrap().push(instance.clone());
            Ok(())
        }
    }

    pub fn remove_instance(&self, instance_uri: &str) -> Result<(), Error> {
        let mut instances = self.instances.write()?;
        let mut selected = self.selected.write()?;
        let position = instances
            .iter()
            .position(|x| x.uri == instance_uri)
            .ok_or(Error::InstanceNotFound)?;
        if instances.len() == 1 {
            return Err(Error::AtLeastOneInstance);
        }
        if let Some(ref selected_instance) = *selected {
            if selected_instance.uri == instance_uri {
                *selected = None;
            }
        }
        instances.remove(position);
        Ok(())
    }

    pub fn select_instance(&self, instance: Option<&Arc<Instance>>) -> Result<(), Error> {
        let mut current = self.selected.write()?;
        if let Some(ref instance) = instance {
            *current = Some(Arc::clone(instance));
        } else {
            *current = None;
        }
        Ok(())
    }

    pub fn select_instance_by_name(&self, instance: &str) -> Result<(), Error> {
        self.select_instance(self.instances().iter().find(|x| x.uri == instance))
    }

    pub fn select_instance_by_index(&self, index: usize) -> Result<(), Error> {
        let instances = self.instances();
        if instances.len() >= index {
            Err(Error::OutOfBounds)
        } else {
            self.select_instance(Some(&instances[index]))
        }
    }

    // API Requests
    pub fn stats(&self) -> Result<StatsResponse, Error> {
        let instance = self.get_instance()?;
        let data: StatsResponse = self
            .client
            .get(&format!("{}/api/v1/stats", instance.uri))?
            .json()?;
        Ok(data)
    }
    pub async fn popular(&self) -> Result<Vec<Content>, Error> {
        let instance = self.get_instance()?;
        println!("{}", &instance.uri);
        let mut data: Vec<Content> = self
            .client
            .get_async(&format!("{}/api/v1/popular", instance.uri))
            .await?
            .json::<Vec<Video>>()
            .await?
            .into_iter()
            .map(|x| Content::Video(x))
            .collect();

        fix_thumbnail_urls(&instance.uri, &mut data);
        Ok(data)
    }

    pub async fn trending(&self) -> Result<Vec<Content>, Error> {
        let instance = self.get_instance()?;
        let mut data: Vec<Content> = self
            .client
            .get_async(&format!("{}/api/v1/trending", instance.uri))
            .await?
            .json::<Vec<Video>>()
            .await?
            .into_iter()
            .map(|x| Content::Video(x))
            .collect();

        fix_thumbnail_urls(&instance.uri, &mut data);

        Ok(data)
    }
}
