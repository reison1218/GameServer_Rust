use super::*;
use crate::room::character::BattleCharacter;
use log::error;
use std::borrow::Borrow;

#[derive(Default)]
pub struct OpenCellRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub battle_data: Option<*const BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

impl OpenCellRobotAction {
    pub fn get_battle_data_ref(&self) -> &BattleData {
        unsafe {
            let ptr = self.battle_data.as_ref().unwrap().as_ref();
            let battle_data_ref = ptr.as_ref().unwrap();
            battle_data_ref
        }
    }

    pub fn new(battle_data: *const BattleData, sender: Sender<RobotTask>) -> Self {
        let mut open_cell = OpenCellRobotAction::default();
        open_cell.battle_data = Some(battle_data);
        open_cell.sender = Some(sender);
        open_cell
    }
}

get_mut_ref!(OpenCellRobotAction);

impl RobotStatusAction for OpenCellRobotAction {
    fn set_sender(&self, sender: Sender<RobotTask>) {
        self.get_mut_ref().sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        self.execute();
    }

    fn execute(&self) {
        let battle_data = self.get_battle_data_ref();
        //校验为配对的地图块数量
        if battle_data.tile_map.un_pair_map.is_empty() {
            return;
        }
        let mut v = Vec::new();
        for key in battle_data.tile_map.un_pair_map.keys() {
            v.push(*key);
        }
        let mut rand = rand::thread_rng();
        let mut index = 0;

        let robot_id = self.robot_id;
        let battle_cter = battle_data.battle_cter.get(&robot_id).unwrap();
        let mut action_type = ActionType::Open.into_u8();

        //剩余翻块次数
        let mut residue_open_times = battle_cter.flow_data.residue_open_times;

        //剩余次数等于0，则啥也不干，直接返回
        if residue_open_times == 0 {
            return;
        }
        //计算可以配对多少个
        let (pair_v, element_index) = cal_pair_num(battle_data, battle_cter);

        //大于1个时，优先配对与自己元素相同的地图块
        if pair_v.len() > 1 {
            if let Some(element_index) = element_index {
                index = element_index;
            } else {
                index = rand.gen_range(0, pair_v.len());
                index = *pair_v.get(index).unwrap();
            }
        } else if pair_v.len() == 1 {
            //翻开能够配对的地图块
            index = *pair_v.get(0).unwrap();
        } else {
            let mut is_cd = false;
            for skill in battle_cter.skills.values() {
                if skill.cd_times > 0_i8 {
                    is_cd = true;
                    break;
                }
            }
            //如果有技能cd的话就随机在地图里面翻开一个地图块
            if is_cd {
                index = rand.gen_range(0, v.len());
            } else {
                //否则
                let res = rand.gen_range(0, 101);
                if res >= 0 && res <= 60 {
                    index = rand.gen_range(0, v.len());
                } else {
                    //跳过turn
                    action_type = ActionType::Skip.into_u8();
                }
            }
        }

        //创建机器人任务执行普通攻击
        let mut robot_task = RobotTask::default();
        robot_task.cmd = action_type;
        let mut map = Map::new();
        map.insert("user_id".to_owned(), Value::from(self.robot_id));
        map.insert("value".to_owned(), Value::from(index));
        map.insert("cmd".to_owned(), Value::from(RoomCode::Action.into_u32()));
        let res = self.sender.as_ref().unwrap().send(robot_task);
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }

    fn exit(&self) {
        unimplemented!()
    }

    fn get_status(&self) -> RobotStatus {
        self.status
    }
}

///计算能够配对的数量
pub fn cal_pair_num(
    battle_data: &BattleData,
    battle_cter: &BattleCharacter,
) -> (Vec<usize>, Option<usize>) {
    let mut index_v = Vec::new();
    let mut element_index: Option<usize> = None;

    //拿到机器人数据
    let robot_data = battle_cter.get_robot_data_ref();
    if let Err(e) = robot_data {
        error!("{:?}", e);
        return (Vec::new(), None);
    }
    let robot_data = robot_data.unwrap();

    //机器人记忆的地图块
    let mut remember_cells = robot_data.remember_map_cell.borrow();

    //这个turn放开的地图块下标
    let open_map_cell_vec = &battle_cter.flow_data.open_map_cell_vec;
    //去掉已经翻开过的,并且添加可以配对的
    for cell_index in open_map_cell_vec.iter() {
        let cell_index = *cell_index;
        let map_cell = battle_data.tile_map.map_cells.get(cell_index).unwrap();
        for re_cell in remember_cells.iter() {
            //如果是已经翻开了的，就跳过
            if cell_index == re_cell.cell_index {
                continue;
            }
            //添加可以配对的
            if map_cell.id == re_cell.cell_id {
                index_v.push(cell_index);
            }
            //添加元素相同的
            if element_index.is_none() && map_cell.element == battle_cter.base_attr.element {
                element_index = Some(map_cell.index);
            }
        }
    }
    (index_v, element_index)
}
