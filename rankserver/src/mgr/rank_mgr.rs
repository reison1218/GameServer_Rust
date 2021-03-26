use std::collections::HashMap;

use super::{RankInfo, RankInfoPtr};
use crate::handler::{modify_nick_name, update_rank, update_season};
use crate::task_timer::Task;
use crate::{REDIS_INDEX_RANK, REDIS_KEY_CURRENT_RANK};
use async_std::task::block_on;
use crossbeam::channel::Sender;
use log::warn;
use rayon::slice::ParallelSliceMut;
use std::collections::hash_map::RandomState;
use tools::cmd_code::RankCode;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

type CmdFn = HashMap<u32, fn(&mut RankMgr, Packet), RandomState>;
///排行榜管理器
#[derive(Default)]
pub struct RankMgr {
    pub rank_vec: Vec<RankInfo>,                //排行榜数据
    pub update_map: HashMap<u32, RankInfoPtr>,  //排行裸指针
    pub cmd_map: CmdFn,                         //命令管理 key:cmd,value:函数指针
    pub need_rank: bool,                        //是否需要排序
    pub last_rank: Vec<RankInfo>,               //上一赛季排行榜数据
    pub user_best_rank: HashMap<u32, RankInfo>, //玩家历史最好排行数据
    sender: Option<TcpSender>,                  //tcp channel的发送方
    pub task_sender: Option<Sender<Task>>,      //任务发送方
}

impl RankMgr {
    pub fn new() -> RankMgr {
        let mut rm = RankMgr::default();
        rm.cmd_init();
        rm
    }

    ///转发到游戏中心服,然后推送给所有特定服务器
    ///比如cmd是游戏服要处理的命令，那么就会推送给全部游戏服
    pub fn push_2_server(&mut self, cmd: u32, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_push_packet_bytes(cmd, user_id, bytes, true, false);
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
        self.rank_vec.par_sort_unstable_by(|a, b| {
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
        self.rank_vec
            .iter_mut()
            .enumerate()
            .for_each(|(index, ri)| {
                if ri.rank != index as i32 && ri.league.id > 0 {
                    ri.rank = index as i32;
                    let user_id = ri.user_id;
                    if need_save {
                        let json_value = serde_json::to_string(ri).unwrap();
                        //持久化到redis
                        let _: Option<String> = redis_lock.hset(
                            REDIS_INDEX_RANK,
                            REDIS_KEY_CURRENT_RANK,
                            user_id.to_string().as_str(),
                            json_value.as_str(),
                        );
                    }
                }
            });
    }
}
