use crate::fsm::status::{
    EnterMineAndDigForNugget, GoHomeAndSleepTilRested, LocationType, Status, StatusAction,
};
use crossbeam::atomic::AtomicCell;
use std::borrow::BorrowMut;
use std::time::Duration;
use tools::macros::GetMutRef;

///矿工结构体
pub struct Miner {
    pub id: AtomicCell<u32>,                     //id
    pub location_type: AtomicCell<LocationType>, //位置
    pub gold_carried: AtomicCell<u32>,           //拥有的金矿数量
    pub money_in_back: AtomicCell<u32>,          //存了多少钱
    pub thirst: AtomicCell<u32>,                 //口渴程度
    pub fatigue: AtomicCell<u32>,                //疲惫程度
    pub status: Box<dyn StatusAction>,           //状态
}

tools::get_mut_ref!(Miner);

impl Default for Box<dyn StatusAction> {
    fn default() -> Self {
        let res = Box::new(EnterMineAndDigForNugget::default());
        res
    }
}

impl Miner {
    pub fn new(id: u32, status: Box<dyn StatusAction>) -> Self {
        let mut miner = Miner::default();
        miner.id.store(id);
        miner.status = status;
        miner
    }

    pub fn set_status(&mut self, status: Box<dyn StatusAction>) {
        self.status = status;
    }
}

impl Default for Miner {
    fn default() -> Self {
        let h = GoHomeAndSleepTilRested {
            status: Status::GoHomeAndSleepTilRested,
        };
        Miner {
            id: AtomicCell::new(0),
            location_type: AtomicCell::new(LocationType::Home),
            gold_carried: AtomicCell::new(0),
            money_in_back: AtomicCell::new(0),
            thirst: AtomicCell::new(0),
            fatigue: AtomicCell::new(0),
            status: Box::new(h),
        }
    }
}

pub trait Robot {
    fn update(&self);

    fn change_status(&self, status: Box<dyn StatusAction>);

    fn get_id(&self) -> u32;

    fn get_status_mut_ref(&self) -> &mut Box<dyn StatusAction>;
}

impl Robot for Miner {
    fn update(&self) {
        std::thread::sleep(Duration::from_secs(2));
        self.status.execute(self);
    }

    ///改变状态函数
    fn change_status(&self, status: Box<dyn StatusAction>) {
        let res = self.get_status_mut_ref();
        res.exit(self);
        //更新状态
        self.get_mut_ref().set_status(status);
        let res = self.get_status_mut_ref();
        //进入新的状态
        res.enter(self);
    }

    fn get_id(&self) -> u32 {
        self.id.load()
    }

    fn get_status_mut_ref(&self) -> &mut Box<dyn StatusAction> {
        self.get_mut_ref().status.borrow_mut()
    }
}

impl Miner {
    pub fn change_location(&self, location_type: LocationType) {
        self.location_type.store(location_type);
        println!("矿工改变位置,前往:{:?}", self.location_type.load());
    }

    pub fn add_gold_carried(&self) {
        self.gold_carried.fetch_add(1);
    }
}
