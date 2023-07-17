use rand::{self, seq::SliceRandom};

use serde::Serialize;
use serde::{self, Deserialize};
use std::io;
use std::sync::RwLock;
use std::sync::{Arc, PoisonError};
use thiserror::Error;
use ureq::{self, Agent, AgentBuilder};

#[derive(Debug, Error)]
pub enum Error {
    #[error("instance already exists")]
    InstanceExists,
    #[error("request failed")]
    UreqError(#[from] ureq::Error),
    #[error("io error")]
    IoError(#[from] io::Error),
    #[error("no known instances")]
    NoInstances,
    #[error("thread with lock panicked")]
    PoisonError,
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
    pub region: String,
    pub open_registrations: bool,
}

#[derive(Debug)]
pub struct InvidiousClient {
    pub instances: RwLock<Instances>,
    pub selected: RwLock<Option<Arc<Instance>>>,
    agent: Agent,
}

// Functions
pub fn fetch_instances() -> Result<Instances, Error> {
    let response: InstancesResponse =
        ureq::get("https://api.invidious.io/instances.json?pretty=1&sort_by=type,users")
            .call()?
            .into_json()?;

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
                    region: instance.region,
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

impl InvidiousClient {
    pub fn new(instances: Instances) -> Self {
        let agent = AgentBuilder::new().https_only(true).build();
        // TODO: Handle a 'no instances available' a little better
        let instances = if instances.len() == 0 {
            vec![Arc::new(Instance {
                uri: "https://vid.puffyan.us".into(),
                region: "US".into(),
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
            agent,
        }
    }

    // Manage Instances
    pub fn get_instance(&self) -> Result<Arc<Instance>, Error> {
        if let Some(ref instance) = *self.selected.read().unwrap() {
            return Ok(instance.clone());
        }

        Ok(self
            .instances
            .read()
            .unwrap()
            .choose(&mut rand::thread_rng())
            .ok_or(Error::NoInstances)?
            .clone())
    }

    pub fn push_instance(&self, instance: Instance) -> Result<(), Error> {
        if self
            .instances
            .read()?
            .iter()
            .position(|x| x.uri == instance.uri)
            .is_some()
        {
            Err(Error::InstanceExists)
        } else {
            self.instances.write().unwrap().push(Arc::new(instance));
            Ok(())
        }
    }

    pub fn remove_instance(&self, instance: Arc<Instance>) -> Result<(), Error> {
        let mut instances = self.instances.write()?;
        let position = instances.iter().position(|x| *x == instance).unwrap();
        instances.remove(position);
        Ok(())
    }

    pub fn select_instance(&self, instance: Option<&Arc<Instance>>) {
        let mut current = self.selected.write().unwrap();
        if let Some(instance) = instance {
            if let Some(ref item) = self
                .instances
                .read()
                .unwrap()
                .iter()
                .find(|&x| x == instance)
            {
                *current = Some(Arc::clone(item));
            } else {
                *current = None
            }
        } else {
            *current = None
        }
    }

    // API Requests
    pub fn stats(&self) -> Result<StatsResponse, Error> {
        let instance = self.get_instance()?;
        let data: StatsResponse = self
            .agent
            .get(format!("{}/api/v1/stats", instance.uri).as_str())
            .call()?
            .into_json()?;
        Ok(data)
    }
}
