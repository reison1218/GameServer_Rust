use serde::{Deserialize, Serialize};
use std::cell::Cell;

///段位结构体
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct League {
    pub id: u8,              //段位id
    pub user_id: u32,        //玩家id
    pub name: String,        //玩家名称
    pub score: u32,          //积分
    pub rank: i32,           //排名
    pub cters: Vec<u32>,     //常用的三个角色
    pub league_time: String, //进入段位时间
    #[serde(skip_serializing)]
    pub version: Cell<u32>, //版本号
}
