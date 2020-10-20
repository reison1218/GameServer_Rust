use crate::battle::battle::RobotCter;
use crate::battle::enums::RobotState;
use crate::goal_ai::cter::Cter;
use crate::handlers::robot_handler::request_robot;
use log::info;
use log::warn;
use rand::Rng;
use std::borrow::Borrow;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use tools::cmd_code::RobotCode;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

///channel管理结构体
#[derive(Default)]
pub struct RobotMgr {
    pub robot_id: AtomicU32,
    pub idle_robot_num: u32,
    pub room_robot_map: HashMap<u64, HashSet<u32>>, //机器人map
    pub robot_map: HashMap<u32, RobotCter>,
    pub cmd_map: HashMap<u32, fn(&RobotMgr, Packet) -> anyhow::Result<()>, RandomState>, //命令管理 key:cmd,value:函数指针
    pub sender: Option<TcpSender>, //tcp channel的发送方
}

tools::get_mut_ref!(RobotMgr);

impl RobotMgr {
    pub fn new() -> Self {
        let mut rm = RobotMgr::default();
        rm.add_robot();
        rm.cmd_map
            .insert(RobotCode::RequestRobot.into_u32(), request_robot);
        rm
    }

    ///将机器人添加到房间
    pub fn add_robot_to_room(&self, room_id: u64, need_num: u32, already_cter: Vec<u32>) {
        //剔除重复到角色
        let cters = [1001, 1002, 1003, 1004, 1005];
        let mut cters_res = cters.clone().to_vec();
        let mut delete_v = Vec::new();
        for i in already_cter.iter() {
            for index in 0..cters.len() {
                let j = cters.get(index).unwrap();
                if i != j {
                    continue;
                }
                delete_v.push(index);
            }
        }
        for index in delete_v {
            cters_res.remove(index);
        }

        //随机角色
        let mut rand = rand::thread_rng();
        let rm = self.get_mut_ref();
        rm.room_robot_map.insert(room_id, HashSet::new());
        let mut hs = rm.room_robot_map.get_mut(&room_id).unwrap();
        let mut need_num = need_num;
        for robot in rm.robot_map.values() {
            if robot.robot_status != RobotState::Idle {
                continue;
            }
            if need_num == 0 {
                break;
            }

            let index = rand.gen_range(0, cters_res.len());
            let cter_id = *cters_res.get(index).unwrap();
            cters_res.remove(index);
            //设置角色id
            robot.base_attr.cter_id.store(cter_id);

            hs.insert(robot.base_attr.robot_id.load());
            rm.idle_robot_num -= 1;
            need_num -= 1;
        }
    }

    ///添加机器人，每次添加1000个
    pub fn add_robot(&self) {
        let self_mut_ref = self.get_mut_ref();
        let size = 1000;
        for i in 0..size {
            let robot_id = self_mut_ref.robot_id.fetch_add(1, Ordering::SeqCst);
            let mut robot = RobotCter::default();
            robot.base_attr.robot_id.store(robot_id);
            self_mut_ref.robot_map.insert(robot_id, robot);
            self_mut_ref.idle_robot_num += 1;
        }
        info!("成功添加{}个机器人！", size);
    }

    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }

    ///执行函数，通过packet拿到cmd，然后从cmdmap拿到函数指针调用
    pub fn invok(&mut self, packet: Packet) {
        let cmd = packet.get_cmd();
        let f = self.cmd_map.get_mut(&cmd);
        if f.is_none() {
            warn!("there is no handler of cmd:{:?}!", cmd);
            return;
        }
        let res: anyhow::Result<()> = f.unwrap()(self, packet);
        match res {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}
