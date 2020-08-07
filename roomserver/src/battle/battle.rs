use crate::battle::battle_enum::{
    ActionUnit, BattleCterState, EffectType, TargetType, TriggerEffectType,
};
use crate::room::character::BattleCharacter;
use crate::room::map_data::{Cell, TileMap};
use crate::room::room::MEMBER_MAX;
use crate::task_timer::Task;
use crate::TEMPLATES;
use log::{error, info, warn};
use protobuf::Message;
use std::collections::HashMap;
use tools::cmd_code::ClientCode;
use tools::protos::base::{ActionUnitPt, BuffPt, TargetPt, TriggerEffectPt};
use tools::protos::battle::S_BATTLE_TURN_NOTICE;
use tools::tcp::TcpSender;
use tools::templates::skill_temp::SkillTemp;

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u32,                        //物品id
    pub skill_temp: &'static SkillTemp, //物品带的技能
}

#[derive(Debug, Clone)]
pub struct Direction {
    pub direction: &'static Vec<i32>,
}

///房间战斗数据封装
#[derive(Clone, Debug)]
pub struct BattleData {
    pub tile_map: TileMap,                          //地图数据
    pub choice_orders: [u32; 4],                    //选择顺序里面放玩家id
    pub next_choice_index: usize,                   //下一个选择的下标
    pub next_turn_index: usize,                     //下个turn的下标
    pub turn_action: ActionUnit,                    //当前回合数据单元封装
    pub turn_orders: [u32; 4],                      //turn行动队列，里面放玩家id
    pub battle_cter: HashMap<u32, BattleCharacter>, //角色战斗数据
    pub turn_limit_time: u64,
    pub task_sender: crossbeam::Sender<Task>, //任务sender
    pub sender: TcpSender,                    //sender
}

impl BattleData {
    pub fn new(task_sender: crossbeam::Sender<Task>, sender: TcpSender) -> Self {
        BattleData {
            tile_map: TileMap::default(),
            choice_orders: [0; 4],
            next_choice_index: 0,
            next_turn_index: 0,
            turn_action: ActionUnit::default(),
            turn_orders: [0; 4],
            battle_cter: HashMap::new(),
            turn_limit_time: 60000, //默认一分钟
            task_sender,
            sender,
        }
    }

    ///下个回合
    pub fn next_turn(&mut self) {
        //计算下一个回合
        self.add_next_turn_index();
        //开始回合触发
        self.turn_start_settlement();
        //给客户端推送战斗turn推送
        self.send_battle_turn_notice();
        //创建战斗turn定时器任务
        self.build_battle_turn_task();
    }

    pub fn add_next_turn_index(&mut self) {
        self.next_turn_index += 1;
        let index = self.next_turn_index;
        if index >= MEMBER_MAX as usize {
            self.next_turn_index = 0;
            return;
        }
        let user_id = self.get_turn_user(Some(index));
        if let Ok(user_id) = user_id {
            if user_id != 0 {
                return;
            }
            self.add_next_turn_index();
        } else {
            warn!("{:?}", user_id.err().unwrap());
        }
    }

    ///翻地图块
    pub fn open_cell(
        &mut self,
        index: usize,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            warn!("{:?}", e);
            anyhow::bail!("")
        }
        let user_id = user_id.unwrap();
        let is_pair;
        unsafe {
            let au_ptr = au as *mut ActionUnitPt;
            let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
            let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
            //处理配对和角色换位置逻辑
            is_pair = self.handler_cell_pair(user_id, index, au_ptr.as_mut().unwrap());

            //处理翻地图块触发buff
            let res =
                self.open_cell_trigger_buff(user_id, index, au_ptr.as_mut().unwrap(), is_pair);
            if let Err(e) = res {
                anyhow::bail!("{:?}", e)
            }

            //处理配对成功与否后的数据
            if is_pair {
                //状态改为可以进行攻击
                battle_cter.is_can_attack = true;
                //如果配对了，则清除上一次翻的地图块
                battle_cter.set_recently_open_cell_index(None);
            } else {
                //更新最近一次翻的下标
                battle_cter.set_recently_open_cell_index(Some(index));
            }

            battle_cter.is_opened_cell = true;
            //处理地图块额外其他的buff
            self.trigger_cell_extra_buff(user_id, index);

            //处理移动后的事件
            let v = self.handler_cter_move(user_id, index);

            //翻块次数-1
            battle_cter.residue_open_times -= 1;

            //玩家技能cd-1
            battle_cter
                .skills
                .values_mut()
                .for_each(|skill| skill.sub_cd(None));

            Ok(Some(v))
        }
    }

    ///处理地图块配对逻辑
    pub unsafe fn handler_cell_pair(
        &mut self,
        user_id: u32,
        index: usize,
        au: &mut ActionUnitPt,
    ) -> bool {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;

        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();

        let cell = self.tile_map.map.get_mut(index).unwrap() as *mut Cell;
        let cell = &mut *cell;
        let mut is_pair = false;
        let cell_id = cell.id;
        au.action_value.push(cell_id);
        let recently_open_cell_index = battle_cter.recently_open_cell_index;
        let mut recently_open_cell_id: Option<u32> = None;
        if let Some(recently_open_cell_index) = recently_open_cell_index {
            let res = self
                .tile_map
                .map
                .get_mut(recently_open_cell_index)
                .unwrap()
                .id;
            recently_open_cell_id = Some(res);
            let last_cell = self
                .tile_map
                .map
                .get_mut(recently_open_cell_index as usize)
                .unwrap() as *mut Cell;
            let last_cell = &mut *last_cell;
            //如果配对了，则修改地图块配对的下标
            if let Some(id) = recently_open_cell_id {
                if cell_id == id {
                    cell.pair_index = Some(recently_open_cell_index as usize);
                    last_cell.pair_index = Some(index);
                    is_pair = true;
                }
            } else {
                is_pair = false;
            }
        }

        //判断改地图块上面有没有角色，有的话将目标位置的玩家挪到操作玩家的位置上
        if cell.user_id > 0 {
            let target_cter = self.battle_cter.get_mut(&cell.user_id).unwrap();
            target_cter.cell_index = battle_cter.cell_index;

            let source_cell = self.tile_map.map.get_mut(battle_cter.cell_index).unwrap();
            source_cell.user_id = target_cter.user_id;
        }
        //改变角色位置
        battle_cter.cell_index = index;
        cell.user_id = battle_cter.user_id;

        if is_pair && recently_open_cell_index.is_some() {
            let mut target_pt = TargetPt::new();
            target_pt.target_type = TargetType::Cell as u32;
            target_pt.target_value = recently_open_cell_index.unwrap() as u32;
            au.targets.push(target_pt);
            info!(
                "user:{} open cell pair! last_cell:{},now_cell:{}",
                battle_cter.user_id,
                recently_open_cell_index.unwrap() as u32,
                index
            );
        }
        is_pair
    }

    ///回合开始触发
    pub fn turn_start_settlement(&mut self) {
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            error!("{:?}", e);
            return;
        }
        let user_id = user_id.unwrap();
        let battle_cter = self.battle_cter.get_mut(&user_id);
        if let None = battle_cter {
            error!("battle_cter is None!user_id:{}", user_id);
            return;
        }
        let battle_cter = battle_cter.unwrap();
        battle_cter.turn_reset();
    }

    ///发送战斗turn推送
    pub fn send_battle_turn_notice(&mut self) {
        let cter = self.get_battle_cter_mut(None);
        if let Err(e) = cter {
            error!("{:?}", e);
            return;
        }
        let cter = cter.unwrap();
        let mut sbtn = S_BATTLE_TURN_NOTICE::new();
        sbtn.user_id = cter.user_id;
        cter.buffs.values().for_each(|buff| {
            let mut buff_pt = BuffPt::new();
            buff_pt.buff_id = buff.id;
            buff_pt.trigger_timesed = buff.trigger_timesed as u32;
            buff_pt.keep_times = buff.keep_times as u32;
            sbtn.buffs.push(buff_pt);
        });

        let bytes = sbtn.write_to_bytes().unwrap();
        for user_id in self.battle_cter.clone().keys() {
            self.send_2_client(ClientCode::BattleTurnNotice, *user_id, bytes.clone());
        }
    }

    ///普通攻击
    /// user_id:发动普通攻击的玩家
    /// targets:被攻击目标
    pub unsafe fn attack(
        &mut self,
        user_id: u32,
        targets: Vec<u32>,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
        let cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let damege = self.calc_damage(user_id);
        let mut aoe_buff: Option<u32> = None;

        cter.buffs
            .values()
            .filter(|buff| buff.id == 4)
            .for_each(|buff| {
                aoe_buff = Some(buff.id);
            });

        let target_user = targets.get(0).unwrap();

        let target_cter = battle_cters.as_mut().unwrap().get_mut(target_user);
        if let None = target_cter {
            let str = format!("there is no cter!user_id:{}", target_user);
            warn!("{:?}", str.as_str());
            anyhow::bail!(str)
        }
        let target_cter = target_cter.unwrap();
        if target_cter.user_id == user_id {
            let str = format!("the attack target can not be Self!user_id:{}", user_id);
            warn!("{:?}", str.as_str());
            anyhow::bail!(str)
        }
        let mut res = damege - target_cter.defence as i32;
        let mut target_pt = TargetPt::new();
        let gd_buff = target_cter.trigger_attack_damge_gd();
        if gd_buff.0 > 0 {
            res = 0;
            let mut te_pt = TriggerEffectPt::new();
            te_pt.set_field_type(TriggerEffectType::Buff as u32);
            te_pt.set_value(gd_buff.0);
            target_pt.passiveEffect.push(te_pt);
        }

        let is_died = target_cter.sub_hp(res);
        if is_died {
            target_cter.state = BattleCterState::Die as u8;
        }

        target_pt.target_type = TargetType::AnyPlayer as u32;
        target_pt.target_value = target_cter.user_id;
        target_pt.effect_type = EffectType::AttackDamage as u32;
        target_pt.effect_value = res as u32;
        //如果有抵挡攻击伤害的buff，并且触发次数为0了
        if gd_buff.0 > 0 && gd_buff.1 {
            target_pt.lost_buffs.push(gd_buff.0);
        } else {
            target_cter.is_attacked = true;
        }
        au.targets.push(target_pt.clone());
        //检查aoebuff
        let target_cter_index = target_cter.cell_index as i32;
        if let Some(buff) = aoe_buff {
            let buff = TEMPLATES.get_buff_ref().get_temp(&buff);
            if let Err(e) = buff {
                warn!("{:?}", e);
                anyhow::bail!("")
            }
            let buff = buff.unwrap();
            let scope_temp = TEMPLATES.get_skill_scope_ref().get_temp(&buff.scope);
            if let Err(e) = scope_temp {
                warn!("{:?}", e);
                anyhow::bail!("")
            }
            let scope_temp = scope_temp.unwrap();

            let res = self.cal_scope(
                user_id,
                target_cter_index as isize,
                TargetType::OtherAnyPlayer,
                None,
                Some(scope_temp),
            );
            if let Err(e) = res {
                error!("{:?}", e);
                anyhow::bail!("")
            }
            let v = res.unwrap();

            //目标周围的玩家
            for user in v {
                if target_cter.user_id == user {
                    continue;
                }
                let cter = self.get_battle_cter_mut(Some(user));
                if let Err(e) = cter {
                    error!("{:?}", e);
                    continue;
                }
                let cter = cter.unwrap();

                let reduce_damage = cter.defence as i32;
                let mut res = damege - reduce_damage;
                let gd_buff = cter.trigger_attack_damge_gd();
                if gd_buff.0 > 0 {
                    res = 0;
                    let mut te_pt = TriggerEffectPt::new();
                    te_pt.set_field_type(TriggerEffectType::Buff as u32);
                    te_pt.set_value(gd_buff.0);
                    target_pt.passiveEffect.push(te_pt);
                }
                let is_died = cter.sub_hp(res);
                if is_died {
                    cter.state = BattleCterState::Die as u8;
                }
                target_pt.target_value = cter.user_id;
                target_pt.effect_value = res as u32;
                if gd_buff.0 > 0 && gd_buff.1 {
                    target_pt.lost_buffs.push(gd_buff.0);
                } else {
                    cter.is_attacked = true;
                }
                au.targets.push(target_pt.clone());
            }
        }
        cter.is_can_attack = false;
        Ok(None)
    }

    ///跳过回合
    pub fn skip_turn(
        &mut self,
        _au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
        //直接下一个turn
        self.next_turn();
        Ok(None)
    }

    ///使用道具,道具都是一次性的，用完了就删掉
    /// user_id:使用道具的玩家
    /// item_id:道具id
    pub fn use_item(
        &mut self,
        user_id: u32,
        item_id: u32,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
        let battle_cter = self.get_battle_cter(Some(user_id)).unwrap();
        let item = battle_cter.items.get(&item_id).unwrap();
        let skill_id = item.skill_temp.id;
        let mut targets = Vec::new();
        targets.push(user_id);
        let res = self.use_skill(user_id, skill_id, targets, au);
        let battle_cter = self.get_battle_cter_mut(Some(user_id)).unwrap();
        //用完了就删除
        battle_cter.items.remove(&item_id);
        res
    }

    ///使用技能
    /// user_id:使用技能的玩家id
    /// target_array目标数组
    pub fn use_skill(
        &mut self,
        user_id: u32,
        skill_id: u32,
        target_array: Vec<u32>,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
        let mut au_vec: anyhow::Result<Option<Vec<ActionUnitPt>>> = Ok(None);
        unsafe {
            //战斗角色
            let battle_cter_ptr =
                self.get_battle_cter_mut(Some(user_id)).unwrap() as *mut BattleCharacter;
            let battle_cter = battle_cter_ptr.as_mut().unwrap();
            //战斗角色身上的技能
            let skill = battle_cter.skills.get_mut(&skill_id).unwrap();
            //校验cd
            if skill.cd_times > 0 {
                let str = format!(
                    "can not use this skill!skill_id:{},cd:{}",
                    skill_id, skill.cd_times
                );
                warn!("{:?}", str.as_str());
                anyhow::bail!(str)
            }
            //技能判定
            let skill_judge = skill.skill_temp.skill_judge;
            if skill_judge != 0 {
                let skill_judge_temp = TEMPLATES.get_skill_judge_ref().get_temp(&(skill_id as u32));
                if let Ok(_skill_judge) = skill_judge_temp {
                    // todo  目前没有判定条件，先留着
                }
            }

            let target = skill.skill_temp.target;
            let target_type = TargetType::from(target);

            //校验目标类型
            let res = self.check_target_array(user_id, target_type, &target_array);
            if let Err(e) = res {
                let str = format!("{:?}", e);
                warn!("{:?}", str.as_str());
                anyhow::bail!(str)
            }

            //换地图块位置
            if [111].contains(&skill_id) {
                if target_array.len() < 2 {
                    let str = format!(
                        "target_array size is error!skill_id:{},user_id:{}",
                        skill_id, user_id
                    );
                    warn!("{:?}", str.as_str());
                    anyhow::bail!(str)
                }
                let source_index = target_array.get(0).unwrap();
                let target_index = target_array.get(1).unwrap();

                let source_index = *source_index as usize;
                let target_index = *target_index as usize;
                au_vec = self.change_index(source_index, target_index, au);
            } else if [20001, 112].contains(&skill_id) {
                //展示地图块
                if target_array.is_empty() {
                    let str = format!(
                        "target_array is empty!skill_id:{},user_id:{}",
                        skill_id, user_id
                    );
                    warn!("{:?}", str.as_str());
                    anyhow::bail!(str)
                }
                let index = *target_array.get(0).unwrap() as usize;
                au_vec = self.show_index(index, au);
            } else if [121, 211, 221, 311, 322, 20002].contains(&skill_id) {
                //上持续性buff
                au_vec = self.add_buff(user_id, skill_id, target_array, au);
            } else if [212].contains(&skill_id) {
                //将1个地图块自动配对。本回合内不能攻击。
                let index = target_array.get(0).unwrap();
                au_vec = self.auto_pair_cell(user_id, *index as usize, au);
            } else if [222].contains(&skill_id) {
                //选择一个玩家，将其移动到一个空地图块上。
                if target_array.len() < 2 {
                    let str = format!(
                        "move_user,the target_array size is error! skill_id:{},user_id:{}",
                        skill_id, user_id
                    );
                    warn!("{:?}", str.as_str());
                    anyhow::bail!(str)
                }
                let target_user = target_array.get(0).unwrap();
                let target_index = target_array.get(1).unwrap();
                au_vec = self.move_user(*target_user, *target_index as usize, au);
            } else if [321].contains(&skill_id) {
                //对你相邻的所有玩家造成1点技能伤害，并回复等于造成伤害值的生命值。
                au_vec = self.skill_damage_and_cure(user_id, battle_cter.cell_index, skill_id, au);
            } else if [411, 421].contains(&skill_id) {
                //造成技能AOE伤害
                au_vec = self.skill_aoe_damage(user_id, skill_id, target_array, au);
            } else if [20004, 20005].contains(&skill_id) {
                let target_user = target_array.get(0).unwrap();
                //单体技能伤害
                au_vec = self.single_skill_damage(skill_id, *target_user, au);
            } else if [20003].contains(&skill_id) {
                //减目标技能cd
                let target_user = target_array.get(0).unwrap();
                au_vec = self.sub_cd(skill_id, *target_user, au);
            }
            //技能cd重制
            skill.reset_cd();
            if let Ok(v) = au_vec {
                if let Some(v) = v {
                    return Ok(Some(v));
                }
            }
            Ok(None)
        }
    }
}
