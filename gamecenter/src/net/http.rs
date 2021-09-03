use crate::Lock;

use async_std::task::block_on;
use http_types::Error as HttpTypesError;
use serde_json::{json, Value};
use tools::http::HttpServerHandler;

pub struct StopAllServerHandler {
    gm: Lock,
}

impl StopAllServerHandler {
    pub fn new(gm: Lock) -> Self {
        StopAllServerHandler { gm }
    }
}

impl HttpServerHandler for StopAllServerHandler {
    fn get_path(&self) -> &str {
        "stop_all"
    }

    fn execute(
        &mut self,
        _: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        let mut lock = block_on(self.gm.lock());
        lock.stop_all_server_handler();
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct KickPlayerHandler {
    gm: Lock,
}

impl KickPlayerHandler {
    pub fn new(gm: Lock) -> Self {
        KickPlayerHandler { gm }
    }
}

impl HttpServerHandler for KickPlayerHandler {
    fn get_path(&self) -> &str {
        "kick"
    }

    fn execute(
        &mut self,
        params: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        if let None = params {
            let value = json!({ "status":"false ","error":"the params is none!" });
            return Ok(value);
        }
        let params = params.unwrap();
        let res = params.as_object();
        if let None = res {
            let value = json!({ "status":"false ","error":"the params is none!" });
            return Ok(value);
        }

        let map = res.unwrap();
        let user_id = map.get("user_id");
        if let None = user_id {
            let value = json!({ "status":"false ","error":"the user_id is none!" });
            return Ok(value);
        }
        let user_id = user_id.unwrap();
        let res = user_id.as_i64();
        if let None = res {
            let value = json!({ "status":"false ","error":"the user_id is not i64!" });
            return Ok(value);
        }
        let user_id = user_id.as_i64().unwrap() as u32;
        let mut lock = block_on(self.gm.lock());
        lock.kick_player_handler(user_id);
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct ReloadTempsHandler {
    gm: Lock,
}

impl ReloadTempsHandler {
    pub fn new(gm: Lock) -> Self {
        ReloadTempsHandler { gm }
    }
}

impl HttpServerHandler for ReloadTempsHandler {
    fn get_path(&self) -> &str {
        "reload_temps"
    }

    fn execute(
        &mut self,
        _: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        let mut lock = block_on(self.gm.lock());
        lock.notice_reload_temps();
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct UpdateSeasonHandler {
    gm: Lock,
}

impl UpdateSeasonHandler {
    pub fn new(gm: Lock) -> Self {
        UpdateSeasonHandler { gm }
    }
}

impl HttpServerHandler for UpdateSeasonHandler {
    fn get_path(&self) -> &str {
        "update_season"
    }

    fn execute(
        &mut self,
        data: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        let mut lock = block_on(self.gm.lock());
        lock.notice_update_season(data.unwrap());
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct UpdateWorldBossHandler {
    gm: Lock,
}

impl UpdateWorldBossHandler {
    pub fn new(gm: Lock) -> Self {
        UpdateWorldBossHandler { gm }
    }
}

impl HttpServerHandler for UpdateWorldBossHandler {
    fn get_path(&self) -> &str {
        "update_world_boss"
    }

    fn execute(
        &mut self,
        data: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        let mut lock = block_on(self.gm.lock());
        lock.notice_update_worldboss(data.unwrap());
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}
