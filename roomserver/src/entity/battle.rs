use super::*;
use crate::entity::character::BattleCharacter;
use std::collections::HashMap;

///回合行为类型
#[derive(Clone, Debug, PartialEq)]
pub enum BattleCterState {
    Alive = 0,
    Die = 1,
}

///回合行为类型
#[derive(Clone, Debug, PartialEq)]
pub enum ActionType {
    None = 0,    //无效值
    Attack = 1,  //普通攻击
    UseItem = 2, //使用道具
    Skip = 3,    //跳过turn
    Open = 4,    //翻块
    Skill = 5,   //使用技能
}

impl From<u32> for ActionType {
    fn from(action_type: u32) -> Self {
        match action_type {
            1 => ActionType::Attack,
            2 => ActionType::UseItem,
            3 => ActionType::Skip,
            4 => ActionType::Open,
            5 => ActionType::Skill,
            _ => ActionType::None,
        }
    }
}

///行动单位
#[derive(Clone, Debug, Default)]
pub struct ActionUnit {
    pub team_id: u32,
    pub user_id: u32,
    pub turn_index: u32,
    pub actions: Vec<Action>,
}

#[derive(Clone, Debug, Default)]
pub struct Action {
    action_type: u8,
    action_value: u32,
}
///房间战斗数据封装
#[derive(Clone, Debug, Default)]
pub struct BattleData {
    pub choice_orders: Vec<u32>,                    //选择顺序里面放玩家id
    pub next_choice_index: usize,                   //下一个选择的下标
    pub next_turn_index: usize,                     //下个turn的下标
    pub turn_action: ActionUnit,                    //当前回合数据单元封装
    pub turn_orders: Vec<u32>,                      //turn行动队列，里面放玩家id
    pub battle_cter: HashMap<u32, BattleCharacter>, //角色战斗数据
}

impl BattleData {
    ///下个回合
    pub fn next_turn(&mut self) {
        //todo 结算上一回合
        self.settlement_last_turn();
        self.next_turn_index += 1;
        if self.next_turn_index > self.turn_orders.len() - 1 {
            self.next_turn_index = 0;
        }
        //todo 开始回合触发
        self.turn_start_trigger();
    }

    ///翻地图块
    pub fn open_cell(&mut self, index: usize) {
        //todo 此处应该计算技能cd
    }

    ///回合开始触发
    pub fn turn_start_trigger(&mut self) {
        //todo
    }

    ///结算上一回合
    pub fn settlement_last_turn(&mut self) {
        //todo
    }

    ///获得玩家回合下标
    pub fn get_turn_index(&self, user_id: u32) -> isize {
        let mut index = 0_isize;
        for member_id in self.turn_orders.iter() {
            if member_id == &user_id {
                return index;
            }
            index += 1;
        }
        return -1;
    }

    ///普通攻击
    pub fn attack(&mut self, user_id: u32, target_id: u32) {
        let damege = self.calc_damage(user_id);
        let reduce_damage = self.calc_reduce_damage(target_id);
        let res = damege - reduce_damage;
        let battle_cter = self.battle_cter.get_mut(&user_id).unwrap();
        battle_cter.hp -= res as i32;
        if battle_cter.hp <= 0 {
            battle_cter.state = BattleCterState::Die as u8;
        }

        //todo 将计算结果推送给客户端
    }

    ///计算伤害
    fn calc_damage(&self, user_id: u32) -> isize {
        let battle_cter = self.battle_cter.get(&user_id).unwrap();
        let damage = battle_cter.atk;
        //todo 此处应该加上角色身上的buff加成
        damage as isize
    }

    ///计算减伤
    fn calc_reduce_damage(&self, user_id: u32) -> isize {
        let battle_cter = self.battle_cter.get(&user_id).unwrap();
        let mut value = battle_cter.defence;
        //todo 此处应该加上角色身上的减伤buff
        value as isize
    }

    //跳过回合
    pub fn skip_turn(&mut self) {
        self.next_choice_index != 1;
        if self.next_turn_index > self.turn_orders.len() {
            self.next_turn_index = 0;
        }
        //返回客户端
    }

    ///校验是否翻过块
    pub fn check_is_open(&self) -> bool {
        if self.turn_action.actions.is_empty() {
            return false;
        }
        for action in self.turn_action.actions.iter() {
            let action_type = ActionType::from(action.action_type as u32);
            if action_type.eq(&ActionType::Open) {
                continue;
            }
            return true;
        }
        return false;
    }
}
