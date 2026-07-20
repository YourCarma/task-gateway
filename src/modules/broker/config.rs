use std::collections::HashSet;

use getset::Getters;
use serde::Deserialize;

use crate::modules::broker::models::TaskType;

#[derive(Clone, Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct BrokerRouteConfig {
    task_type: TaskType,
    exchange: String,
    service_name: String,
}

impl BrokerRouteConfig {
    pub fn new(
        task_type: impl Into<String>,
        exchange: impl Into<String>,
        service_name: impl Into<String>,
    ) -> Self {
        Self {
            task_type: TaskType::new(task_type),
            exchange: exchange.into(),
            service_name: service_name.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct MessageBrokerConfig {
    address: String,
    routes: Vec<BrokerRouteConfig>,
}

impl MessageBrokerConfig {
    pub fn new(address: impl Into<String>, routes: Vec<BrokerRouteConfig>) -> Self {
        Self {
            address: address.into(),
            routes,
        }
    }

    pub fn route(&self, task_type: &TaskType) -> Option<&BrokerRouteConfig> {
        self.routes
            .iter()
            .find(|route| route.task_type() == task_type)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.routes.is_empty() {
            return Err("broker.routes must contain at least one route".to_string());
        }

        let mut task_types = HashSet::with_capacity(self.routes.len());
        for (index, route) in self.routes.iter().enumerate() {
            if route.task_type().as_str().trim().is_empty() {
                return Err(format!(
                    "broker.routes[{index}].task_type must not be empty"
                ));
            }
            if route.exchange().trim().is_empty() {
                return Err(format!("broker.routes[{index}].exchange must not be empty"));
            }
            if route.service_name().trim().is_empty() {
                return Err(format!(
                    "broker.routes[{index}].service_name must not be empty"
                ));
            }
            if route.service_name().contains(':') {
                return Err(format!(
                    "broker.routes[{index}].service_name must not contain ':'"
                ));
            }
            if !task_types.insert(route.task_type()) {
                return Err(format!(
                    "duplicate broker route for task_type '{}'",
                    route.task_type()
                ));
            }
        }

        Ok(())
    }
}
