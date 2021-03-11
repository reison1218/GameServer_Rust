use crate::battle::battle::BattleData;
use crate::battle::battle_enum::buff_type::{
    ADD_ATTACK_AND_AOE, MANUAL_MOVE_AND_PAIR_ADD_ENERGY, PAIR_SAME_ELEMENT_ADD_ATTACK,
    RESET_MAP_ADD_ATTACK,
};
use crate::battle::battle_enum::{BattleCterState, SkillConsumeType};
use crate::battle::battle_enum::{TargetType, TRIGGER_SCOPE_NEAR_TEMP_ID};
use crate::battle::battle_skill::Skill;
use crate::battle::battle_trigger::TriggerEvent;
use crate::robot::robot_trigger::RobotTriggerType;
use crate::room::character::BattleCharacter;
use crate::room::map_data::MapCellType;
use crate::room::map_data::TileMap;
use crate::room::RoomType;
use crate::TEMPLATES;
use log::{error, warn};
use protobuf::Message;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Deref;
use tools::cmd_code::ClientCode;
use tools::protos::base::{ActionUnitPt, SummaryDataPt};
use tools::protos::battle::S_SUMMARY_NOTICE;
use tools::protos::server_protocol::B_S_SUMMARY;

use super::mission::{trigger_mission, MissionTriggerType};

impl BattleData {
    ///处理战斗结算核心逻辑，不管地图刷新逻辑
    /// 返回一个元组类型：是否结算，存活玩家数量，第一名的玩家列表
    pub fn summary(&mut self) -> Vec<B_S_SUMMARY> {
        let allive_count = self
            .battle_cter
            .values()
            .filter(|x| x.status.state == BattleCterState::Alive)
            .count();

        //回客户端消息
        let mut ssn = S_SUMMARY_NOTICE::new();
        let mut need_summary = false;
        let mut bgs = Vec::new();
        let (leave_user, punishment) = self.leave_user;

        //如果房间就只有最后一个人了，直接计算
        if allive_count <= 1 {
            need_summary = true;
            //如果达到结算条件，则进行结算
            let self_ptr = self as *mut BattleData;
            unsafe {
                let self_mut = self_ptr.as_mut().unwrap();
                for member_cter in self.battle_cter.values_mut() {
                    if member_cter.is_died() {
                        continue;
                    }
                    member_cter.status.state = BattleCterState::Die;
                    let user_id = member_cter.get_user_id();
                    self_mut.after_cter_died_trigger(user_id, true, false);
                }
            }
        } else if leave_user > 0 {
            //如果有玩家退出房间
            need_summary = true;
        }
        if need_summary {
            for spa_v in self.summary_vec.iter_mut() {
                for su in spa_v.iter_mut() {
                    let smp: SummaryDataPt = su.clone().into();
                    ssn.summary_datas.push(smp.clone());
                    if !su.push_to_server {
                        let mut bg = B_S_SUMMARY::new();
                        bg.set_room_type(self.room_type.into_u32());
                        bg.set_summary_data(smp);
                        bgs.push(bg);
                        su.push_to_server = true;
                    }
                }
            }
        }

        if ssn.summary_datas.len() > 0 {
            let res = ssn.write_to_bytes();
            match res {
                Ok(bytes) => {
                    let v = self.get_battle_cters_vec();
                    for member_id in v {
                        //强退的人不发
                        if member_id == leave_user && punishment {
                            continue;
                        }
                        self.send_2_client(ClientCode::SummaryNotice, member_id, bytes.clone());
                    }
                    self.leave_user = (0, false);
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
        bgs
    }

    ///使用道具,道具都是一次性的，用完了就删掉
    /// user_id:使用道具的玩家
    /// item_id:道具id
    pub fn use_item(
        &mut self,
        user_id: u32,
        item_id: u32,
        targets: Vec<u32>,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
        let battle_cter = self.get_battle_cter(Some(user_id), true);
        if let Err(e) = battle_cter {
            error!("{:?}", e);
            anyhow::bail!("")
        }
        let battle_cter = battle_cter.unwrap();
        let item = battle_cter.items.get(&item_id);
        if let None = item {
            error!("item is None!user_id:{},item_id:{}", user_id, item_id);
            anyhow::bail!("")
        }
        let item = item.unwrap();
        let skill_id = item.skill_temp.id;
        let res = self.use_skill(user_id, skill_id, true, targets, au)?;
        let battle_cter = self.get_battle_cter_mut(Some(user_id), true);
        if let Err(e) = battle_cter {
            error!("{:?}", e);
            anyhow::bail!("")
        }
        let battle_cter = battle_cter.unwrap();
        //用完了就删除
        battle_cter.items.remove(&item_id);
        Ok(res)
    }

    ///跳过回合
    pub fn skip_turn(&mut self, _au: &mut ActionUnitPt) {
        //直接下一个turn
        self.next_turn();
    }

    ///使用技能
    /// user_id:使用技能的玩家id
    /// target_array目标数组
    pub fn use_skill(
        &mut self,
        user_id: u32,
        skill_id: u32,
        is_item: bool,
        target_array: Vec<u32>,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
        let mut au_vec: Option<Vec<ActionUnitPt>> = None;
        unsafe {
            //战斗角色
            let res = self.get_battle_cter_mut(Some(user_id), true);
            if let Err(e) = res {
                error!("{:?}", e);
                anyhow::bail!("")
            }
            let res = res.unwrap();
            let battle_cter_ptr = res as *mut BattleCharacter;
            let battle_cter = battle_cter_ptr.as_mut().unwrap();
            //战斗角色身上的技能
            let mut skill_s;
            let skill;
            if is_item {
                let res = TEMPLATES.skill_temp_mgr().get_temp(&skill_id);
                if let Err(e) = res {
                    error!("{:?}", e);
                    anyhow::bail!("")
                }
                let skill_temp = res.unwrap();
                skill_s = Some(Skill::from(skill_temp));
                skill = skill_s.as_mut().unwrap();
            } else {
                skill = battle_cter.skills.get_mut(&skill_id).unwrap();
                let res = self.before_use_skill_trigger(user_id);
                if let Err(e) = res {
                    warn!("{:?}", e);
                    anyhow::bail!("")
                }
            }

            let target = skill.skill_temp.target;
            let target_type = TargetType::try_from(target as u8).unwrap();

            //技能判定
            let skill_judge = skill.skill_temp.skill_judge as u32;
            let skill_function_id = skill.function_id;

            //校验目标类型
            let res = self.check_target_array(user_id, target_type, &target_array);
            if let Err(e) = res {
                warn!("{:?}", e);
                anyhow::bail!("")
            }

            //校验技能可用判定条件
            if skill_judge > 0 {
                self.check_skill_judge(user_id, skill_judge, Some(skill_id), None)?;
            }

            //根据技能id去找函数指针里面的函数，然后进行执行
            let self_ptr = self as *mut BattleData;
            for skill_function_ids in self_ptr.as_ref().unwrap().skill_function_cmd_map.keys() {
                if !skill_function_ids.deref().contains(&skill_function_id) {
                    continue;
                }
                let fn_ptr = self
                    .skill_function_cmd_map
                    .get_mut(skill_function_ids.deref())
                    .unwrap();
                au_vec = fn_ptr(self, user_id, skill_id, target_array, au);
                break;
            }

            //如果不是用能量的，则用完技能之后重制cd,把cd加上
            if skill.skill_temp.consume_type != SkillConsumeType::Energy as u8 {
                if skill.cd_times <= 0 {
                    skill.reset_cd();
                }
            } else {
                let mut v = skill.skill_temp.consume_value as i8;
                v = v * -1;
                battle_cter.add_energy(v);
            }
            //使用技能后触发
            self.after_use_skill_trigger(user_id, skill_id, is_item, au);
        }
        Ok(au_vec)
    }

    ///普通攻击
    /// user_id:发动普通攻击的玩家
    /// targets:被攻击目标
    pub unsafe fn attack(
        &mut self,
        user_id: u32,
        targets: Vec<u32>,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<()> {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
        let cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let mut aoe_buff: Option<u32> = None;

        //塞选出ape的buff
        cter.battle_buffs
            .buffs
            .values()
            .filter(|buff| ADD_ATTACK_AND_AOE.contains(&buff.function_id))
            .for_each(|buff| {
                aoe_buff = Some(buff.get_id());
            });

        let index = targets.get(0).unwrap();
        let target_cter = self.get_battle_cter_mut_by_map_cell_index(*index as usize);

        if let Err(e) = target_cter {
            warn!("{:?}", e);
            anyhow::bail!("")
        }

        let target_cter = target_cter.unwrap();
        let target_user_id = target_cter.get_user_id();
        let target_user_index = target_cter.get_map_cell_index() as u32;
        if target_user_id == user_id {
            warn!("the attack target can not be Self!user_id:{}", user_id);
            anyhow::bail!("")
        }
        if target_cter.is_died() {
            warn!("the target is died!user_id:{}", target_cter.get_user_id());
            anyhow::bail!("")
        }
        let mut is_last_one = false;

        if aoe_buff.is_none() {
            is_last_one = true;
        }

        //扣血
        let target_pt = self.deduct_hp(user_id, target_user_id, None, is_last_one);

        if let Err(e) = target_pt {
            error!("{:?}", e);
            anyhow::bail!("")
        }
        let mut target_pt = target_pt.unwrap();
        if target_pt.effects.is_empty() {
            error!("target_pt's effects is empty!");
            anyhow::bail!("")
        }
        //目标被攻击，触发目标buff
        if target_pt.effects.get(0).unwrap().effect_value > 0 {
            self.attacked_trigger(target_user_id, &mut target_pt);
        }
        au.targets.push(target_pt);
        //检查aoebuff
        if let Some(buff) = aoe_buff {
            let buff = TEMPLATES.buff_temp_mgr().get_temp(&buff);
            if let Err(e) = buff {
                warn!("{:?}", e);
                anyhow::bail!("")
            }
            let scope_temp = TEMPLATES
                .skill_scope_temp_mgr()
                .get_temp(&TRIGGER_SCOPE_NEAR_TEMP_ID);
            if let Err(e) = scope_temp {
                warn!("{:?}", e);
                anyhow::bail!("")
            }
            let scope_temp = scope_temp.unwrap();
            let (_, v) = self.cal_scope(
                user_id,
                target_user_index as isize,
                TargetType::OtherAnyPlayer,
                None,
                Some(scope_temp),
            );

            //目标周围的玩家
            for index in 0..v.len() {
                let target_user = v.get(index).unwrap();
                let target_user = *target_user;
                if target_user_id == target_user {
                    continue;
                }
                if index == v.len() - 1 {
                    is_last_one = true;
                }
                //扣血
                let target_pt = self.deduct_hp(user_id, target_user, None, is_last_one);
                match target_pt {
                    Ok(mut target_pt) => {
                        //目标被攻击，触发目标buff
                        self.attacked_trigger(target_user, &mut target_pt);
                        au.targets.push(target_pt);
                    }
                    Err(e) => error!("{:?}", e),
                }
            }
        }
        cter.pair_attack_reward_movement_points();
        //触发翻地图块任务
        trigger_mission(
            self,
            user_id,
            vec![MissionTriggerType::Attack],
            1,
            (target_user_id, 0),
        );
        Ok(())
    }

    ///刷新地图
    pub fn reset_map(
        &mut self,
        room_type: RoomType,
        season_id: u32,
        last_map_id: u32,
    ) -> anyhow::Result<()> {
        //地图刷新前触发buff
        self.before_map_refresh_buff_trigger();
        let allive_count = self
            .battle_cter
            .values()
            .filter(|x| x.status.state == BattleCterState::Alive)
            .count();
        let res = TileMap::init(room_type, season_id, allive_count as u8, last_map_id)?;
        self.last_map_id = res.id;
        self.tile_map = res;
        self.reflash_map_turn = Some(self.next_turn_index);
        unsafe {
            let mut buff_function_id;
            let mut buff_id;
            //刷新角色状态和触发地图刷新的触发buff
            for cter in self.battle_cter.values_mut() {
                if cter.is_died() {
                    continue;
                }
                cter.round_reset();
                let cter_ptr = cter as *mut BattleCharacter;
                for buff in cter_ptr.as_mut().unwrap().battle_buffs.buffs.values() {
                    buff_function_id = buff.function_id;
                    buff_id = buff.get_id();
                    //刷新地图增加攻击力
                    if RESET_MAP_ADD_ATTACK.contains(&buff_function_id) {
                        cter.trigger_add_damage_buff(buff_id);
                    }
                    //匹配相同元素的地图块加攻击，在地图刷新的时候，攻击要减回来
                    if PAIR_SAME_ELEMENT_ADD_ATTACK.contains(&buff_function_id) {
                        cter.battle_buffs.add_damage_buffs.remove(&buff_id);
                    }
                }
            }
        }
        Ok(())
    }

    ///翻地图块
    pub fn open_map_cell(
        &mut self,
        index: usize,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            warn!("{:?}", e);
            anyhow::bail!("open_map_cell fail!")
        }
        let user_id = user_id.unwrap();
        let str = format!(
            "open_map_cell fail!user_id:{},index:{}",
            user_id, self.next_turn_index
        );
        let mut is_pair = false;
        unsafe {
            let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
            let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id);
            if let None = battle_cter {
                error!("battle_cter is not find!user_id:{}", user_id);
                anyhow::bail!("{:?}", str.as_str())
            }
            let battle_cter = battle_cter.unwrap();

            //先移动
            let v = self.handler_cter_move(user_id, index, au);
            if let Err(e) = v {
                warn!("{:?}", e);
                anyhow::bail!("{:?}", str.as_str())
            }

            let (is_died, v) = v.unwrap();
            //判断玩家死了没
            if is_died {
                return Ok(Some(v));
            }
            //减去移动点数
            battle_cter.flow_data.residue_movement_points -= 1;
            //玩家技能cd-1
            battle_cter.sub_skill_cd(None);
            //设置是否可以结束turn状态
            battle_cter.set_is_can_end_turn(true);

            //判断是否商店
            let map_cell = self.tile_map.map_cells.get(index).unwrap();
            let is_market = map_cell.cell_type == MapCellType::MarketCell;
            if !is_market {
                //打开地图块
                self.exec_open_map_cell(user_id, index);

                //再配对
                is_pair = self.handler_map_cell_pair(user_id);
            }
            let mut buff_function_id;
            let mut buff_id;
            //消耗移动点干点什么，配对了又干点什么
            for buff in battle_cter.battle_buffs.buffs.values() {
                buff_function_id = buff.function_id;
                buff_id = buff.get_id();
                //移动加能量，配对加能量
                if MANUAL_MOVE_AND_PAIR_ADD_ENERGY.contains(&buff_function_id) {
                    self.manual_move_and_pair(Some(user_id), user_id, buff_id, is_pair, au);
                }
            }

            //如果是商店不要往下走
            if is_market {
                return Ok(Some(v));
            }

            //处理翻地图块触发buff
            let res = self.open_map_cell_trigger(user_id, au, is_pair);
            if let Err(e) = res {
                anyhow::bail!("{:?}", e)
            }

            let robot_trigger_type;
            if is_pair {
                robot_trigger_type = RobotTriggerType::MapCellPair;
            } else {
                robot_trigger_type = RobotTriggerType::SeeMapCell;
            }
            //调用触发器
            self.map_cell_trigger_for_robot(index, robot_trigger_type);
            Ok(Some(v))
        }
    }

    ///下个turn
    pub fn next_turn(&mut self) {
        //本回合结束
        self.turn_end();
        //计算下一个回合
        self.add_next_turn();
        //给客户端推送战斗turn推送
        self.send_battle_turn_notice();
        //创建战斗turn定时器任务
        self.build_battle_turn_task();
    }

    ///本turn结束,结算一些回合结束该干的事情
    pub fn turn_end(&mut self) {
        //清空翻开地图玩家id
        self.clear_open_cells();
        let cter = self.get_battle_cter_mut(None, true);
        if cter.is_err() {
            return;
        }
        let cter = cter.unwrap();
        //turn结束重制
        cter.turn_end_reset();
    }

    ///回合开始触发
    pub fn turn_start_summary(&mut self) {
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            error!("{:?}", e);
            return;
        }
        let battle_data = self as *mut BattleData;
        let user_id = user_id.unwrap();
        unsafe {
            let self_mut = self;
            //结算玩家身上的buff
            for cter in self_mut.battle_cter.values_mut() {
                for buff in cter.battle_buffs.buffs.values() {
                    //如果是永久buff,则跳过
                    if buff.permanent {
                        continue;
                    }
                    let buff_id = buff.get_id();
                    battle_data.as_mut().unwrap().consume_buff(
                        buff_id,
                        Some(cter.get_user_id()),
                        None,
                        true,
                    );
                }
            }

            //结算该玩家加在地图块上的buff
            for map_cell in self_mut.tile_map.map_cells.iter_mut() {
                for buff in map_cell.buffs.values() {
                    if buff.permanent {
                        continue;
                    }
                    let buff_id = buff.get_id();
                    battle_data.as_mut().unwrap().consume_buff(
                        buff_id,
                        None,
                        Some(map_cell.index),
                        true,
                    );
                }
            }

            //容错处理，如果没有地图块可以翻了，就允许不翻块的情况下结束turn
            if user_id > 0 {
                let mut is_can_skip_turn: bool = true;
                for &index in self_mut.tile_map.un_pair_map.keys() {
                    let map_cell = self_mut.tile_map.map_cells.get(index);
                    if let None = map_cell {
                        continue;
                    }
                    let map_cell = map_cell.unwrap();
                    if map_cell.check_is_locked() {
                        continue;
                    }
                    is_can_skip_turn = false;
                    break;
                }
                let battle_cter = self_mut.get_battle_cter_mut(Some(user_id), true);
                if let Err(e) = battle_cter {
                    error!("{:?}", e);
                    return;
                }
                //turn结算玩家
                let battle_cter = battle_cter.unwrap();
                battle_cter.turn_start_reset();
                battle_cter.set_is_can_end_turn(is_can_skip_turn);
            }
        }
    }
}
