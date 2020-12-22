use crate::battle::battle::BattleData;
use crate::battle::battle_enum::buff_type::{
    ADD_ATTACK_AND_AOE, PAIR_SAME_ELEMENT_ADD_ATTACK, RESET_MAP_ADD_ATTACK,
};
use crate::battle::battle_enum::{AttackState, BattleCterState, SkillConsumeType};
use crate::battle::battle_enum::{TargetType, TRIGGER_SCOPE_NEAR_TEMP_ID};
use crate::battle::battle_skill::Skill;
use crate::battle::battle_trigger::TriggerEvent;
use crate::robot::robot_trigger::RobotTriggerType;
use crate::room::character::BattleCharacter;
use crate::room::map_data::TileMap;
use crate::room::room_model::RoomType;
use crate::TEMPLATES;
use log::{error, warn};
use protobuf::Message;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Deref;
use tools::cmd_code::ClientCode;
use tools::protos::base::{ActionUnitPt, SummaryDataPt};
use tools::protos::battle::S_SUMMARY_NOTICE;
use tools::protos::server_protocol::R_G_SUMMARY;

impl BattleData {
    ///处理战斗结算，不管地图刷新逻辑
    /// 返回一个元组类型：是否结算，存活玩家数量，第一名的玩家列表
    pub fn battle_summary(&mut self) -> Option<R_G_SUMMARY> {
        let allive_count = self
            .battle_cter
            .values()
            .filter(|x| x.status.state == BattleCterState::Alive)
            .count();
        //如果达到结算条件，则进行结算
        if allive_count <= 1 {
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

            //回客户端消息
            let mut ssn = S_SUMMARY_NOTICE::new();
            let mut index = self.rank_vec.len();
            if index == 0 {
                return None;
            } else {
                index -= 1;
            }
            let mut rgs = R_G_SUMMARY::new();
            for members in self.rank_vec.get(index) {
                for sp in members.iter() {
                    let mut smp = SummaryDataPt::new();
                    smp.user_id = sp.user_id;
                    smp.cter_id = sp.cter_id;
                    smp.rank = sp.rank as u32;
                    smp.grade = sp.grade as u32;
                    smp.reward_score = sp.reward_score;
                    smp.league_score = sp.league_score;
                    ssn.summary_datas.push(smp.clone());
                    rgs.summary_datas.push(smp);
                }
            }
            let res = ssn.write_to_bytes();
            match res {
                Ok(bytes) => {
                    let v = self.get_battle_cters_vec();
                    for member_id in v {
                        self.send_2_client(ClientCode::SummaryNotice, member_id, bytes.clone());
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
            return Some(rgs);
        }
        None
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
                let res = TEMPLATES.get_skill_temp_mgr_ref().get_temp(&skill_id);
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
            for skill_ids in self_ptr.as_ref().unwrap().skill_cmd_map.keys() {
                if !skill_ids.deref().contains(&skill_id) {
                    continue;
                }
                let fn_ptr = self.skill_cmd_map.get_mut(skill_ids.deref()).unwrap();
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
            .filter(|buff| ADD_ATTACK_AND_AOE.contains(&buff.id))
            .for_each(|buff| {
                aoe_buff = Some(buff.id);
            });

        let index = targets.get(0).unwrap();
        let target_cter = self.get_battle_cter_mut_by_map_cell_index(*index as usize);

        if let Err(e) = target_cter {
            warn!("{:?}", e);
            anyhow::bail!("")
        }

        let target_cter = target_cter.unwrap();
        let target_user_id = target_cter.get_user_id();
        let target_user_index = target_cter.get_map_cell_index();
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
            self.attacked_buffs_trigger(target_user_id, &mut target_pt);
        }
        au.targets.push(target_pt);
        //检查aoebuff
        if let Some(buff) = aoe_buff {
            let buff = TEMPLATES.get_buff_temp_mgr_ref().get_temp(&buff);
            if let Err(e) = buff {
                warn!("{:?}", e);
                anyhow::bail!("")
            }
            let scope_temp = TEMPLATES
                .get_skill_scope_temp_mgr_ref()
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
                        self.attacked_buffs_trigger(target_user, &mut target_pt);
                        au.targets.push(target_pt);
                    }
                    Err(e) => error!("{:?}", e),
                }
            }
        }
        cter.status.attack_state = AttackState::None;
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
            //刷新角色状态和触发地图刷新的触发buff
            for cter in self.battle_cter.values_mut() {
                if cter.is_died() {
                    continue;
                }
                cter.round_reset();
                let cter = cter as *mut BattleCharacter;
                for buff in cter.as_mut().unwrap().battle_buffs.buffs.values_mut() {
                    //刷新地图增加攻击力
                    if RESET_MAP_ADD_ATTACK.contains(&buff.id) {
                        cter.as_mut().unwrap().trigger_add_damage_buff(buff.id);
                    }
                    //匹配相同元素的地图块加攻击，在地图刷新的时候，攻击要减回来
                    if PAIR_SAME_ELEMENT_ADD_ATTACK.contains(&buff.id) {
                        cter.as_mut()
                            .unwrap()
                            .battle_buffs
                            .add_damage_buffs
                            .remove(&buff.id);
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
        let is_pair;
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
            let v = v.unwrap();
            //判断玩家死了没
            if battle_cter.is_died() {
                return Ok(Some(v));
            }
            //再配对
            is_pair = self.handler_map_cell_pair(user_id, None);

            //处理翻地图块触发buff
            let res = self.open_map_cell_buff_trigger(user_id, au, is_pair);
            if let Err(e) = res {
                anyhow::bail!("{:?}", e)
            }

            //更新翻的地图块下标
            battle_cter.flow_data.open_map_cell_vec.push(index);
            //翻块次数-1
            battle_cter.flow_data.residue_open_times -= 1;

            //玩家技能cd-1
            battle_cter.skills.values_mut().for_each(|skill| {
                if !skill.is_active {
                    skill.add_cd(-1)
                }
            });
            battle_cter.set_is_can_end_turn(true);
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
        //清空翻开地图玩家id
        self.clear_open_cells();
        //计算下一个回合
        self.add_next_turn_index();
        //给客户端推送战斗turn推送
        self.send_battle_turn_notice();
        //创建战斗turn定时器任务
        self.build_battle_turn_task();
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
        if user_id > 0 {
            let battle_cter = self.get_battle_cter_mut(Some(user_id), true);
            if let Ok(battle_cter) = battle_cter {
                battle_cter.turn_reset();
            }
        }

        unsafe {
            //结算玩家身上的buff
            for cter in battle_data.as_mut().unwrap().battle_cter.values_mut() {
                for buff in cter.battle_buffs.buffs.values() {
                    //如果是永久buff,则跳过
                    if buff.permanent {
                        continue;
                    }
                    let buff_id = buff.id;
                    battle_data.as_mut().unwrap().consume_buff(
                        buff_id,
                        Some(cter.get_user_id()),
                        None,
                        true,
                    );
                }
            }

            //结算该玩家加在地图块上的buff
            for map_cell in battle_data.as_mut().unwrap().tile_map.map_cells.iter_mut() {
                for buff in map_cell.buffs.values() {
                    if buff.permanent {
                        continue;
                    }
                    let buff_id = buff.id;
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
                for index in battle_data.as_mut().unwrap().tile_map.un_pair_map.keys() {
                    let map_cell = battle_data.as_mut().unwrap().tile_map.map_cells.get(*index);
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
                let battle_cter = self.get_battle_cter_mut(Some(user_id), true);
                if let Err(e) = battle_cter {
                    error!("{:?}", e);
                    return;
                }
                battle_cter.unwrap().set_is_can_end_turn(is_can_skip_turn);
            }
        }
    }
}
