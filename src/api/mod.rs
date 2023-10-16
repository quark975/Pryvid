use enum_dispatch::enum_dispatch;
use futures::future::join_all;
use isahc::prelude::*;
use isahc::{config::RedirectPolicy, http::StatusCode, HttpClient};
use lazy_static::lazy_static;
use rand::{self, seq::SliceRandom};
use regex::Regex;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
use std::io;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::{Duration, Instant};
use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("Instance already exists")]
    InstanceExists,
    #[error("Request failed")]
    RequestError(#[from] isahc::Error),
    #[error("IO error")]
    IoError(#[from] io::Error),
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
    #[error("Instance returned bad status code")]
    BadStatusCode,
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
    pub has_trending: Option<bool>,
    pub has_popular: Option<bool>,
    pub open_registrations: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[enum_dispatch]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "video")]
    Video(Video),
    #[serde(rename = "playlist")]
    Playlist(Playlist),
    #[serde(rename = "channel")]
    Channel(Channel),
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Video {
    pub title: String,
    #[serde(rename = "videoId")]
    pub id: String,
    #[serde(rename = "viewCount")]
    pub views: u64,
    #[serde(rename = "lengthSeconds")]
    pub length: u32,
    #[serde(rename = "videoThumbnails")]
    pub thumbnails: Vec<Thumbnail>,
    pub author: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
    #[serde(rename = "publishedText")]
    pub published: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Playlist {
    pub title: String,
    #[serde(rename = "playlistId")]
    pub id: String,
    pub author: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
    #[serde(rename = "videoCount")]
    pub video_count: u64,
    #[serde(rename = "playlistThumbnail")]
    pub thumbnail: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Channel {
    #[serde(rename = "author")]
    pub title: String,
    #[serde(rename = "authorId")]
    pub id: String,
    #[serde(rename = "authorThumbnails")]
    pub thumbnails: Vec<Thumbnail>,
    #[serde(rename = "subCount")]
    pub subscribers: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Thumbnail {
    pub quality: Option<String>,
    #[serde(rename = "url")]
    pub uri: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DetailedVideo {
    // Video Info
    pub title: String,
    #[serde(rename = "videoId")]
    pub id: String,
    pub description: String,
    #[serde(rename = "descriptionHtml")]
    pub description_html: String,
    #[serde(rename = "publishedText")]
    pub published: String,
    #[serde(rename = "lengthSeconds")]
    pub length: u32,
    pub author: String,
    #[serde(rename = "authorId")]
    pub author_id: String,

    // Statistics
    #[serde(rename = "viewCount")]
    pub views: u64,
    #[serde(rename = "likeCount")]
    pub likes: u32,
    #[serde(rename = "dislikeCount")]
    pub dislikes: u32,
    #[serde(rename = "subCountText")]
    pub subscribers: String,

    // Thumbnails
    #[serde(rename = "videoThumbnails")]
    pub thumbnails: Vec<Thumbnail>,
    #[serde(rename = "authorThumbnails")]
    pub author_thumbnails: Vec<Thumbnail>,

    // Media
    #[serde(rename = "formatStreams")]
    pub format_streams: Vec<FormatStream>,
    pub captions: Vec<Caption>,

    // Recommended
    #[serde(rename = "recommendedVideos")]
    pub recommended: Vec<Video>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FormatStream {
    #[serde(rename = "url")]
    pub uri: String,
    pub quality: String,
    pub fps: u32,
    pub resolution: String,
    pub size: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Caption {
    pub label: String,
    pub language_code: String,
    #[serde(rename = "url")]
    pub uri: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DetailedChannel {
    #[serde(rename = "author")]
    pub title: String,
    #[serde(rename = "authorId")]
    pub id: String,
    #[serde(rename = "authorThumbnails")]
    pub thumbnails: Vec<Thumbnail>,
    #[serde(rename = "authorBanners")]
    pub banners: Vec<Thumbnail>,
    #[serde(rename = "subCount")]
    pub subscribers: u64,
    #[serde(rename = "totalViews")]
    pub total_views: u128,
    pub description: String,
    #[serde(rename = "descriptionHtml")]
    pub description_html: String,
    #[serde(rename = "authorVerified")]
    pub verified: bool,
    #[serde(rename = "latestVideos")]
    pub videos: Vec<Video>,
    #[serde(rename = "relatedChannels")]
    pub related_channels: Vec<Channel>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DetailedPlaylist {
    pub title: String,
    #[serde(rename = "playlistId")]
    pub id: String,
    pub author: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
    #[serde(rename = "videoCount")]
    pub video_count: u64,
    #[serde(rename = "playlistThumbnail")]
    pub thumbnail: String,
    pub videos: Vec<Video>,
}

#[derive(Debug)]
pub struct InvidiousClient {
    instances: RwLock<Instances>,
    selected: RwLock<Option<Arc<Instance>>>,
}

// Global
lazy_static! {
    static ref HTTP_CLIENT: HttpClient = HttpClient::builder()
        .timeout(Duration::from_secs(10))
        .redirect_policy(RedirectPolicy::Limit(10))
        .build()
        .unwrap();
}

// Functions
pub async fn fetch_instances() -> Result<Instances, Error> {
    let response: Vec<(String, InstanceResponse)> = HTTP_CLIENT
        .get_async("https://api.invidious.io/instances.json?pretty=1&sort_by=type,users")
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
                        has_trending: None,
                        has_popular: None,
                    })),
                }))
            } else {
                None
            }
        })
        .collect();

    // Ping in batches of 8 at a time
    // TODO: find a way to improve this maybe
    for instances in instances.chunks(8) {
        join_all(instances.iter().map(|x| x.update_info())).await;
    }

    Ok(instances)
}

fn format_input_uri(uri: &str) -> String {
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

fn correct_uri(instance_uri: &str, uri: &str) -> String {
    if uri.starts_with("/vi/") {
        // If domain isn't present (i.e. /vi/lcIObyvI3uw/maxres.jpg)
        format!("{}{}", instance_uri, uri)
    } else if uri.starts_with("//") {
        // If protocol isn't present (i.e. //yt3.googleusercontent.com/ytc/...)
        format!("https:{}", uri)
    } else {
        uri.to_string()
    }
}

#[enum_dispatch(Content)]
pub trait CorrectUri {
    fn correct_uri(&mut self, instance: &Instance) {
        if let Some(thumbnail) = self.thumbnail().cloned() {
            self.thumbnail()
                .replace(&mut correct_uri(&instance.uri, &thumbnail));
        }
        if let Some(x) = self.thumbnails() {
            x.iter_mut()
                .for_each(|y| y.uri = correct_uri(&instance.uri, &y.uri))
        }
        if let Some(x) = self.channels() {
            x.iter_mut().for_each(|y| y.correct_uri(instance))
        }
        if let Some(x) = self.videos() {
            x.iter_mut().for_each(|y| y.correct_uri(instance))
        }
    }
    fn thumbnail(&mut self) -> Option<&mut String> {
        None
    }
    fn thumbnails(&mut self) -> Option<&mut [Thumbnail]> {
        None
    }
    fn channels(&mut self) -> Option<&mut [Channel]> {
        None
    }
    fn videos(&mut self) -> Option<&mut [Video]> {
        None
    }
}

impl CorrectUri for Video {
    fn thumbnails(&mut self) -> Option<&mut [Thumbnail]> {
        Some(self.thumbnails.as_mut_slice())
    }
}

impl CorrectUri for DetailedVideo {
    fn thumbnails(&mut self) -> Option<&mut [Thumbnail]> {
        Some(self.thumbnails.as_mut_slice())
    }
}

impl CorrectUri for Channel {
    fn thumbnails(&mut self) -> Option<&mut [Thumbnail]> {
        Some(self.thumbnails.as_mut_slice())
    }
}

impl CorrectUri for DetailedChannel {
    fn thumbnails(&mut self) -> Option<&mut [Thumbnail]> {
        Some(self.thumbnails.as_mut_slice())
    }
    fn channels(&mut self) -> Option<&mut [Channel]> {
        Some(self.related_channels.as_mut_slice())
    }
    fn videos(&mut self) -> Option<&mut [Video]> {
        Some(self.videos.as_mut_slice())
    }
}

impl CorrectUri for Playlist {
    fn thumbnail(&mut self) -> Option<&mut String> {
        Some(&mut self.thumbnail)
    }
}

impl CorrectUri for DetailedPlaylist {
    fn thumbnail(&mut self) -> Option<&mut String> {
        Some(&mut self.thumbnail)
    }
    fn videos(&mut self) -> Option<&mut [Video]> {
        Some(self.videos.as_mut_slice())
    }
}

impl Instance {
    pub async fn from_uri(uri: &str) -> Result<Instance, Error> {
        let uri = format_input_uri(uri);
        let response: StatsResponse = HTTP_CLIENT
            .get_async(format!("{}/api/v1/stats", &uri))
            .await?
            .json()
            .await?;
        let instance = Instance {
            uri,
            info: Arc::new(RwLock::new(InstanceInfo {
                has_trending: None,
                has_popular: None,
                open_registrations: response.open_registrations,
            })),
        };
        instance.update_info().await?;
        Ok(instance)
    }

    pub async fn update_info(&self) -> Result<(), Error> {
        let mut response = HTTP_CLIENT
            .get_async(format!("{}/api/v1/popular", self.uri))
            .await?;
        let has_popular = response.json::<Vec<Value>>().await.is_ok();

        let mut response = HTTP_CLIENT
            .get_async(format!("{}/api/v1/trending", self.uri))
            .await?;
        let has_trending = response.json::<Vec<Value>>().await.is_ok();

        let mut info = self.info.write()?;
        info.has_popular = Some(has_popular);
        info.has_trending = Some(has_trending);

        Ok(())
    }

    pub async fn ping(&self, endpoint: Option<&str>) -> Result<u128, Error> {
        let elapsed = Instant::now();
        let response = HTTP_CLIENT
            .get_async(format!("{}{}", self.uri, endpoint.unwrap_or("/")))
            .await?;
        if response.status() == StatusCode::OK {
            let elapsed = elapsed.elapsed();
            Ok(elapsed.as_millis())
        } else {
            println!("{}: {}", self.uri, response.status());
            Err(Error::BadStatusCode)
        }
    }

    // Data Requests
    pub async fn stats(&self) -> Result<StatsResponse, Error> {
        let mut response = HTTP_CLIENT
            .get_async(&format!("{}/api/v1/stats", self.uri))
            .await?;
        if response.status() == StatusCode::OK {
            Ok(response.json::<StatsResponse>().await?)
        } else {
            Err(Error::BadStatusCode)
        }
    }

    async fn fetch_video_page(&self, endpoint: &str) -> Result<Vec<Content>, Error> {
        let mut response = HTTP_CLIENT
            .get_async(&format!("{}{}", self.uri, endpoint))
            .await?;

        if response.status() == StatusCode::OK {
            let mut data: Vec<Content> = response
                .json::<Vec<Video>>()
                .await?
                .into_iter()
                .map(Content::Video)
                .collect();

            for item in data.iter_mut() {
                item.correct_uri(self);
            }
            Ok(data)
        } else {
            Err(Error::BadStatusCode)
        }
    }

    pub async fn popular(&self) -> Result<Vec<Content>, Error> {
        self.fetch_video_page("/api/v1/popular").await
    }

    pub async fn trending(&self) -> Result<Vec<Content>, Error> {
        self.fetch_video_page("/api/v1/trending").await
    }

    pub async fn video(&self, video_id: &str) -> Result<DetailedVideo, Error> {
        let mut response = HTTP_CLIENT
            .get_async(&format!("{}/api/v1/videos/{}", self.uri, video_id))
            .await?;

        if response.status() == StatusCode::OK {
            let mut data: DetailedVideo = response.json().await?;

            for video in &mut data.recommended {
                video.correct_uri(self);
            }

            Ok(data)
        } else {
            Err(Error::BadStatusCode)
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Content>, Error> {
        let mut response = HTTP_CLIENT
            .get_async(&format!(
                "{}/api/v1/search?q={}",
                self.uri,
                urlencoding::encode(query)
            ))
            .await?;

        if response.status() == StatusCode::OK {
            let mut data: Vec<Content> = response.json().await?;

            for item in data.iter_mut() {
                item.correct_uri(self);
            }
            Ok(data)
        } else {
            Err(Error::BadStatusCode)
        }
    }

    pub async fn channel(&self, id: &str) -> Result<DetailedChannel, Error> {
        let mut response = HTTP_CLIENT
            .get_async(&format!("{}/api/v1/channels/{}", self.uri, id))
            .await?;

        if response.status() == StatusCode::OK {
            let mut data: DetailedChannel = response.json().await?;
            data.correct_uri(self);
            Ok(data)
        } else {
            Err(Error::BadStatusCode)
        }
    }

    pub async fn channel_playlists(&self, id: &str) -> Result<Vec<Playlist>, Error> {
        let mut response = HTTP_CLIENT
            .get_async(&format!("{}/api/v1/channels/{}/playlists", self.uri, id))
            .await?;
        if response.status() == StatusCode::OK {
            let data = serde_json::from_value::<Vec<Playlist>>(
                response.json::<Value>().await?["playlists"].take(),
            )
            .unwrap();
            Ok(data)
        } else {
            Err(Error::BadStatusCode)
        }
    }

    pub async fn playlist(&self, id: &str) -> Result<DetailedPlaylist, Error> {
        let mut response = HTTP_CLIENT
            .get_async(&format!("{}/api/v1/playlists/{}", self.uri, id))
            .await?;

        if response.status() == StatusCode::OK {
            let mut data: DetailedPlaylist = response.json().await?;
            data.correct_uri(self);
            Ok(data)
        } else {
            Err(Error::BadStatusCode)
        }
    }
}

impl InvidiousClient {
    pub fn new(instances: Instances) -> Self {
        // TODO: Handle a 'no instances available' a little better
        let instances = if instances.is_empty() {
            vec![Arc::new(Instance {
                uri: "https://vid.puffyan.us".into(),
                info: Arc::new(RwLock::new(InstanceInfo {
                    has_trending: Some(true),
                    has_popular: Some(true),
                    open_registrations: true,
                })),
            })]
        } else {
            instances
        };

        InvidiousClient {
            selected: RwLock::new(None),
            instances: RwLock::new(instances),
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
        false
    }
    pub fn is_added(&self, instance: &Arc<Instance>) -> bool {
        let instances = self.instances();
        instances.iter().any(|x| x.uri == instance.uri)
    }

    pub fn get_instance(&self) -> Arc<Instance> {
        if let Some(ref instance) = *self.selected.read().unwrap() {
            instance.clone()
        } else {
            self.instances()
                .choose(&mut rand::thread_rng())
                .unwrap() // Guaranteeing that this never fails saves a lot on error handling
                .clone()
        }
    }

    pub fn get_trending_instance(&self) -> Result<Arc<Instance>, Error> {
        Ok(self
            .instances()
            .into_iter()
            .filter(|x| x.info.read().unwrap().has_trending.unwrap_or(false))
            .collect::<Vec<Arc<Instance>>>()
            .choose(&mut rand::thread_rng())
            .ok_or(Error::InstanceNotFound)?
            .clone())
    }

    pub fn get_popular_instance(&self) -> Result<Arc<Instance>, Error> {
        Ok(self
            .instances()
            .into_iter()
            .filter(|x| x.info.read().unwrap().has_popular.unwrap_or(false))
            .collect::<Vec<Arc<Instance>>>()
            .choose(&mut rand::thread_rng())
            .ok_or(Error::InstanceNotFound)?
            .clone())
    }

    pub fn push_instance(&self, instance: Arc<Instance>) -> Result<(), Error> {
        if self.instances().iter().any(|x| x.uri == instance.uri) {
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
        if let Some(instance) = instance {
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
}
