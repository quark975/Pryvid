use rand::{self, seq::SliceRandom};

use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use serde::{self, Deserialize};
use serde_json::Value;
use std::io;
use std::sync::RwLock;
use std::sync::{Arc, PoisonError};
use thiserror::Error;
use isahc::HttpClient;
use isahc::prelude::*;

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

type InstancesResponse = Vec<(String, InstanceResponse)>;

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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub uri: String,
    pub has_trending: bool,
    pub has_popular: bool,
    pub open_registrations: bool,
}

#[derive(Debug)]
pub struct InvidiousClient {
    instances: RwLock<Instances>,
    selected: RwLock<Option<Arc<Instance>>>,
    client: HttpClient,
}

// Functions
pub fn fetch_instances() -> Result<Instances, Error> {
    let response: InstancesResponse =
        isahc::get("https://api.invidious.io/instances.json?pretty=1&sort_by=type,users")?
        .json()?;

    Ok(response
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
                    open_registrations,
                    has_trending: false,
                    has_popular: false,
                }))
            } else {
                None
            }
        })
        .collect())
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
        let response: StatsResponse = isahc::get(format!("{}/api/v1/stats", &uri))?
            .json()?;
        let mut instance = Instance {
            uri,
            has_trending: false,
            has_popular: false,
            open_registrations: response.open_registrations
        };
        instance.update_info();
        Ok(instance)
    }

    pub fn update_info(&mut self) {
        let response = isahc::get(format!("{}/api/v1/popular", self.uri));
        if let Ok(mut response) = response {
            self.has_popular = response.json::<Vec<Value>>().is_ok();
        }

        let response = isahc::get(format!("{}/api/v1/trending", self.uri));
        if let Ok(mut response) = response {
            self.has_trending = response.json::<Vec<Value>>().is_ok();
        }
    }
}

impl InvidiousClient {
    pub fn new(instances: Instances) -> Self {
        let client = HttpClient::new().unwrap();
        // TODO: Handle a 'no instances available' a little better
        let instances = if instances.len() == 0 {
            vec![Arc::new(Instance {
                uri: "https://vid.puffyan.us".into(),
                has_trending: true,
                has_popular: true,
                open_registrations: true,
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
            return Arc::ptr_eq(selected_instance, instance)
        }
        return false;
    }
    pub fn is_added(&self, instance: &Arc<Instance>) -> bool {
        let instances = self.instances();
        instances.iter().position(|x| x.uri == instance.uri).is_some()
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

    pub fn remove_instance(&self, instance: Arc<Instance>) -> Result<(), Error> {
        let mut instances = self.instances.write()?;
        let mut selected = self.selected.write()?;
        let position = instances.iter().position(|x| *x == instance).unwrap();
        if let Some(ref selected_instance) = *selected {
            if *selected_instance == instance {
                *selected = None;
            }
        }
        instances.remove(position);
        Ok(())
    }

    pub fn select_instance(&self, instance: Option<&Arc<Instance>>) -> Result<(), Error> {
        let mut current = self.selected.write()?;
        if let Some(instance) = instance {
            if let Some(ref item) = self.instances().iter().find(|&x| x == instance) {
                *current = Some(Arc::clone(item));
            } else {
                *current = None;
            }
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
            .get(format!("{}/api/v1/stats", instance.uri).as_str())?
            .json()?;
        Ok(data)
    }
}
