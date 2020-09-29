use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

///pos操作类型
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Status{
    None=0,
    HpLess=1,
    SkillCDOK=2,
    ResidueOpenTimes=3,
    AttackAbleTo=4,
    EnergyEnough=5,
}

impl Default for Status{
    fn default() -> Self {
        Status::None
    }
}

///战斗角色数据
#[derive(Clone, Debug, Default)]
pub struct BattleCharacter{
    pub user_id: u32,   //玩家id
    pub cter_id: u32,   //角色的配置id
    pub grade: u8,      //等级
    pub atk: u8,        //攻击力
    pub hp: i16,        //角色血量
    pub defence: u8,    //角色防御
    pub energy: u8,     //角色能量
    pub max_energy: u8, //能量上限
    pub element: u8,    //角色元素
    pub hp_max: i16,    //血上限
    pub item_max: u8,   //道具数量上限
    pub status:Status,  //状态
}



pub trait UpateStatus {
    fn update_status(&mut self,status:Status);
}

pub trait Start:UpateStatus{
    fn run(&mut self){
        loop{
            self.check();
            self.execute();
        }
    }
    fn check(&mut self);

    fn execute(&mut self);
}

pub trait Attack{
    fn attack(&mut self);
}

impl Start for BattleCharacter{
    fn check(&mut self) {
        if self.status == Status::None{
            let status = Status::AttackAbleTo;
            self.status = status;
            self.execute();
        }
    }

    fn execute(&mut self) {
        self.attack();
    }
}

impl UpateStatus for BattleCharacter{
    fn update_status(&mut self,status:Status) {
        self.status = status;
    }
}

impl Attack for BattleCharacter{
    fn attack(&mut self) {
    }
}




fn main() {
    let mut bc = BattleCharacter::default();
    bc.run();
}
