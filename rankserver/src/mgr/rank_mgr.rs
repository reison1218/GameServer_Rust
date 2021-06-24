use std::collections::HashMap;

use super::RankInfo;
use crate::handler::{modify_nick_name, update_rank, update_season};
use crate::task_timer::Task;
use crate::{REDIS_INDEX_RANK, REDIS_KEY_CURRENT_RANK};
use async_std::task::block_on;
use crossbeam::channel::Sender;
use log::warn;
use rayon::slice::ParallelSliceMut;
use std::collections::hash_map::RandomState;
use tools::cmd_code::RankCode;
use tools::tcp_message_io::TcpHandler;
use tools::util::packet::Packet;

type CmdFn = HashMap<u32, fn(&mut RankMgr, Packet), RandomState>;
///排行榜管理器
#[derive(Default)]
pub struct RankMgr {
    pub rank_vec: Vec<RankInfo>,                //排行榜数据
    pub cmd_map: CmdFn,                         //命令管理 key:cmd,value:函数指针
    pub need_rank: bool,                        //是否需要排序
    pub last_rank: Vec<RankInfo>,               //上一赛季排行榜数据
    pub user_best_rank: HashMap<u32, RankInfo>, //玩家历史最好排行数据
    tcp_handler: Option<TcpHandler>,            //tcp channel的发送方
    pub task_sender: Option<Sender<Task>>,      //任务发送方
}

impl RankMgr {
    pub fn new() -> RankMgr {
        let mut rm = RankMgr::default();
        rm.cmd_init();
        rm
    }

    pub fn get_rank_mut(&mut self, user_id: u32) -> Option<&mut RankInfo> {
        for ri in self.rank_vec.iter_mut() {
            if ri.user_id != user_id {
                continue;
            }
            return Some(ri);
        }
        None
    }

    ///转发到游戏中心服,然后推送给所有特定服务器
    ///比如cmd是游戏服要处理的命令，那么就会推送给全部游戏服
    pub fn push_2_server(&self, cmd: u32, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_push_packet_bytes(cmd, user_id, bytes, true, false);
        let tcp_handler = self.tcp_handler.as_ref().unwrap();
        let endpoint = tcp_handler.endpoint;
        tcp_handler
            .node_handler
            .network()
            .send(endpoint, bytes.as_slice());
    }

    pub fn set_sender(&mut self, tcp_handler: TcpHandler) {
        self.tcp_handler = Some(tcp_handler);
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        //更新排行榜
        self.cmd_map
            .insert(RankCode::UpdateRank.into_u32(), update_rank);
        //更新赛季
        self.cmd_map
            .insert(RankCode::UpdateSeasonPush.into_u32(), update_season);
        //修改名字
        self.cmd_map
            .insert(RankCode::ModifyNickName.into_u32(), modify_nick_name);
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

impl RankMgr {
    pub fn sort(&mut self, need_save: bool) {
        if !self.need_rank {
            return;
        }
        self.rank_vec.par_sort_by(|a, b| {
            //如果段位等级一样
            if a.league.get_league_id() == b.league.get_league_id() {
                if a.league.league_time != b.league.league_time {
                    //看时间
                    return a.league.league_time.cmp(&b.league.league_time);
                }
            }
            //段位不一样直接看分数
            b.get_score().cmp(&a.get_score())
        });
        let mut redis_lock = block_on(crate::REDIS_POOL.lock());
        for (index, ri_mut) in self.rank_vec.iter_mut().enumerate() {
            let user_id = ri_mut.user_id;
            let rank = ri_mut.rank;
            let league_id = ri_mut.league.id;
            let index = index as i32;
            if rank != index && league_id > 0 {
                ri_mut.rank = index;
                if need_save {
                    let json_value = serde_json::to_string(ri_mut).unwrap();
                    //持久化到redis
                    let _: Option<u32> = redis_lock.hset(
                        REDIS_INDEX_RANK,
                        REDIS_KEY_CURRENT_RANK,
                        user_id.to_string().as_str(),
                        json_value.as_str(),
                    );
                }
            }
        }
    }
}
