use crate::status::{GoHomeAndSleepTilRested, LocationType, Status, StatusAction};
use std::borrow::BorrowMut;
use std::time::Duration;

///战斗角色数据
pub struct BattleCharacter {
    pub location_type: LocationType,   //位置
    pub gold_carried: u32,             //拥有的金矿数量
    pub money_in_back: u32,            //存了多少钱
    pub thirst: u32,                   //口渴程度
    pub fatigue: u32,                  //疲惫程度
    pub status: Box<dyn StatusAction>, //状态
}

impl Default for BattleCharacter {
    fn default() -> Self {
        let h = GoHomeAndSleepTilRested {
            status: Status::GoHomeAndSleepTilRested,
        };
        BattleCharacter {
            location_type: LocationType::Home,
            gold_carried: 0,
            money_in_back: 0,
            thirst: 0,
            fatigue: 0,
            status: Box::new(h),
        }
    }
}

pub trait Robot {
    fn update_status(&mut self);

    fn change_status(&mut self, status: Box<dyn StatusAction>);

    fn get_id(&self) -> u32;
}

impl Robot for BattleCharacter {
    fn update_status(&mut self) {
        std::thread::sleep(Duration::from_secs(2));
    }

    ///改变状态函数
    fn change_status(&mut self, status: Box<dyn StatusAction>) {
        self.update_status();
        //退出当前状态
        unsafe {
            let mut b_ptr = self as *mut BattleCharacter;
            let b = b_ptr.as_mut().unwrap();
            b_ptr.as_mut().unwrap().status.exit(b);
            //更新状态
            b.status = status;
            //进入新的状态
            b.status.enter(self);
        }
    }

    fn get_id(&self) -> u32 {
        0
    }
}

impl BattleCharacter {
    pub fn change_location(&mut self, location_type: LocationType) {
        self.location_type = location_type;
        println!("矿工改变位置,前往:{:?}", self.location_type);
    }

    pub fn add_gold_carried(&mut self) {
        self.gold_carried += 1;
    }
}
