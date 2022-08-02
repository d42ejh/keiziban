use async_graphql::{Error, Result, SimpleObject};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use jsonwebtoken::{encode, EncodingKey, Header};
use num_traits::{FromPrimitive, ToPrimitive};
use systemstat::{platform::PlatformImpl, Platform, System};
use tracing::{event, Level};
use uuid::Uuid;

pub struct SystemInfoContext {
    sys: PlatformImpl,
    interval: std::time::Duration,
    last_update: Option<std::time::SystemTime>,
    last_info: Option<SystemInfo>,
}

impl SystemInfoContext {
    pub fn new(interval: std::time::Duration) -> Self {
        let sys = System::new();

        SystemInfoContext {
            sys: sys,
            interval: interval,
            last_update: None,
            last_info: None,
        }
    }

    fn actually_get_info(&mut self) -> anyhow::Result<SystemInfo> {
        let mem = self.sys.memory()?;
        event!(Level::DEBUG, "Memory:\n{:?}", mem);
        let info = SystemInfo {
            total_mem_available: mem.total.as_u64() as usize,
            free_mem: mem.free.as_u64() as usize,
        };
        Ok(info)
    }

    /// Get system informations.
    pub fn get_info(&mut self) -> anyhow::Result<SystemInfo> {
        let now = std::time::SystemTime::now();
        if self.last_update.is_some() && self.last_update.unwrap() + self.interval > now {
            event!(Level::DEBUG, "No need to update");
            debug_assert!(self.last_info.is_some());
            return Ok(self.last_info.unwrap());
        }
        event!(Level::DEBUG, "Update system info");
        let info = self.actually_get_info()?;
        self.last_info = Some(info);

        self.last_update = Some(now);
        Ok(self.last_info.unwrap())
    }
}

#[derive(SimpleObject, Clone, Copy)]
pub struct SystemInfo {
    pub total_mem_available: usize,
    pub free_mem: usize,
}
