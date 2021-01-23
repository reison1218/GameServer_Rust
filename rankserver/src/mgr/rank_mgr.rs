use std::collections::HashMap;

use super::{RankInfo, RankInfoPtr};
use crate::handler::{get_rank, update_rank, update_season};
use crate::task_timer::Task;
use crossbeam::channel::Sender;
use log::warn;
use std::collections::hash_map::RandomState;
use tools::cmd_code::{RankCode, ServerCommonCode};
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

type CmdFn = HashMap<u32, fn(&mut RankMgr, Packet) -> anyhow::Result<()>, RandomState>;
///排行榜管理器
#[derive(Default)]
pub struct RankMgr {
    pub rank_vec: Vec<RankInfo>, //排行榜数据
    pub update_map: HashMap<u32, RankInfoPtr>,
    pub cmd_map: CmdFn,                    //命令管理 key:cmd,value:函数指针
    pub need_rank: bool,                   //是否需要排序
    sender: Option<TcpSender>,             //tcp channel的发送方
    pub task_sender: Option<Sender<Task>>, //任务发送方
}

impl RankMgr {
    pub fn new() -> RankMgr {
        let mut rm = RankMgr::default();
        rm.cmd_init();
        rm
    }

    pub fn set_task_sender(&mut self, sender: Sender<Task>) {
        self.task_sender = Some(sender);
    }

    ///转发到游戏中心服,然后推送给所有特定服务器
    ///比如cmd是游戏服要处理的命令，那么就会推送给全部游戏服
    pub fn push_2_server(&mut self, cmd: u32, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_push_packet_bytes(cmd, user_id, bytes, true, false);
        self.sender.as_mut().unwrap().send(bytes);
    }

    ///转发到游戏中心服
    pub fn send_2_server_direction(
        &mut self,
        cmd: u32,
        user_id: u32,
        bytes: Vec<u8>,
        server_token: u32,
    ) {
        let bytes =
            Packet::build_packet_bytes_direction(cmd, user_id, bytes, true, false, server_token);
        self.sender.as_mut().unwrap().send(bytes);
    }

    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        //更新排行榜
        self.cmd_map
            .insert(RankCode::UpdateRank.into_u32(), update_rank);
        //更新赛季
        self.cmd_map
            .insert(ServerCommonCode::UpdateSeason.into_u32(), update_season);
        //获得排行榜
        self.cmd_map.insert(RankCode::GetRank.into_u32(), get_rank);
    }

    ///执行函数，通过packet拿到cmd，然后从cmdmap拿到函数指针调用
    pub fn invok(&mut self, packet: Packet) {
        let cmd = packet.get_cmd();
        let f = self.cmd_map.get_mut(&cmd);
        if f.is_none() {
            warn!("there is no handler of cmd:{:?}!", cmd);
            return;
        }
        let _ = f.unwrap()(self, packet);
    }
}
