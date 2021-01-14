use std::{cmp::Ordering, collections::HashMap, time::Duration};

use super::{RankInfo, RankInfoPtr};
use crate::handler::update_rank;
use log::warn;
use rayon::slice::ParallelSliceMut;
use std::collections::hash_map::RandomState;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

type CmdFn = HashMap<u32, fn(&mut RankMgr, Packet) -> anyhow::Result<()>, RandomState>;
///排行榜管理器
#[derive(Default)]
pub struct RankMgr {
    pub rank_vec: Vec<RankInfo>, //排行榜数据
    pub update_map: HashMap<u32, RankInfoPtr>,
    pub cmd_map: CmdFn,        //命令管理 key:cmd,value:函数指针
    sender: Option<TcpSender>, //tcp channel的发送方
}

impl RankMgr {
    pub fn new() -> RankMgr {
        let mut rm = RankMgr::default();
        rm.cmd_init();
        rm
    }

    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        //更新排行榜
        self.cmd_map.insert(123, update_rank);
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

    ///更新排行榜
    pub fn update_rank_info(&mut self) {
        let mut a = RankInfo::new(1, "1".to_owned());
        a.league.league_time = 123456;
        a.league.league_score = 1;
        self.rank_vec.push(a);
        std::thread::sleep(Duration::from_secs(1));
        let mut b = RankInfo::new(2, "2".to_owned());
        b.league.league_time = 123456;
        b.league.league_score = 2;
        self.rank_vec.push(b);
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
        })
    }
}
