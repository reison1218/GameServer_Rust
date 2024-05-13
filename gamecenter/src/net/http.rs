use std::collections::HashMap;

use crate::Lock;

use async_std::task::block_on;
use serde_json::json;
use tools::{
    http::HttpServerHandler,
    json::{JsonValue, JsonValueTrait},
};

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
        "/stop_all"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
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
        "/kick"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
        let _json_params = JsonValue::from_bytes(_json_params);
        if let Err(err) = _json_params {
            log::warn!("{:?}", err);
            return Ok(json!(r#"{"result": "fail!","errMessage":"参数有问题!"}"#));
        }
        let params = _json_params.unwrap();

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
        "/reload_temps"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
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
        "/update_season"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
        log::info!("收到modify_server,uri:{:?}", _uri);
        let _json_params = JsonValue::from_bytes(_json_params);
        if let Err(err) = _json_params {
            log::warn!("{:?}", err);
            return Ok(json!(r#"{"result": "fail!","errMessage":"参数有问题!"}"#));
        }
        let params = _json_params.unwrap();

        let mut lock = block_on(self.gm.lock());
        lock.notice_update_season(params);
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
        "/update_world_boss"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<JsonValue> {
        let _json_params = JsonValue::from_bytes(_json_params);
        if let Err(err) = _json_params {
            log::warn!("{:?}", err);
            return Ok(json!(r#"{"result": "fail!","errMessage":"参数有问题!"}"#));
        }
        let params = _json_params.unwrap();
        let mut lock = block_on(self.gm.lock());
        lock.notice_update_worldboss(params);
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}
