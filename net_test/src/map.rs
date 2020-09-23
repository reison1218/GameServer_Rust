use std::env;
use std::collections::{HashMap, HashSet};
use tools::templates::tile_map_temp::TileMapTempMgr;
use rand::{random, Rng};
use std::borrow::Borrow;
use tools::templates::template::TemplatesMgr;
use tools::templates::buff_temp::BuffTemp;
use crate::TEMPLATES;
use std::slice::Iter;

pub fn generate_map(){
    TileMap::init(4,Some(false)).unwrap();
}

///目标类型枚举
#[derive(Clone, Debug, PartialEq)]
pub enum TargetType {
    None = 0,            //无效目标
    Cell = 1,            //地图块
    AnyPlayer = 2,       //任意玩家
    PlayerSelf = 3,      //玩家自己
    AllPlayer = 4,       //所有玩家
    OtherAllPlayer = 5,  //除自己外所有玩家
    OtherAnyPlayer = 6,  //除自己外任意玩家
    UnOpenCell = 7,      //未翻开的地图块
    UnPairCell = 8,      //未配对的地图块
    NullCell = 9,        //空的地图块，上面没人
    UnPairNullCell = 10, //未配对的地图块
    CellPlayer = 11,     //地图块上的玩家
}

impl From<u8> for TargetType {
    fn from(value: u8) -> Self {
        match value {
            1 => TargetType::Cell,
            2 => TargetType::AnyPlayer,
            3 => TargetType::PlayerSelf,
            4 => TargetType::AllPlayer,
            5 => TargetType::OtherAllPlayer,
            6 => TargetType::OtherAnyPlayer,
            7 => TargetType::UnOpenCell,
            8 => TargetType::UnPairCell,
            9 => TargetType::NullCell,
            10 => TargetType::UnPairNullCell,
            11 => TargetType::CellPlayer,
            _ => TargetType::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Direction {
    pub direction: &'static Vec<isize>,
}

#[derive(Clone, Debug)]
pub struct Buff {
    pub id: u32,
    pub buff_temp: &'static BuffTemp,
    pub trigger_timesed: i8,   //已经触发过的次数
    pub keep_times: i8,        //剩余持续轮数
    pub scope: Vec<Direction>, //buff的作用范围
    pub permanent: bool,       //是否永久
    pub user_id: u32,          //来源的玩家id
}

impl Buff {
    pub fn get_target(&self) -> TargetType {
        let target_type = TargetType::from(self.buff_temp.target);
        target_type
    }
    

    pub(crate) fn sub_trigger_timesed(&mut self) {
        self.trigger_timesed -= 1;
        if self.trigger_timesed < 0 {
            self.trigger_timesed = 0;
        }
    }

    pub(crate) fn sub_keep_times(&mut self) {
        self.keep_times -= 1;
        if self.keep_times < 0 {
            self.keep_times = 0;
        }
    }
}

impl From<&'static BuffTemp> for Buff {
    fn from(bt: &'static BuffTemp) -> Self {
        let mut b = Buff {
            id: bt.id,
            trigger_timesed: bt.trigger_times as i8,
            keep_times: bt.keep_time as i8,
            buff_temp: bt,
            scope: Vec::new(),
            permanent: bt.keep_time == 0 && bt.trigger_times == 0,
            user_id: 0,
        };
        let mut v = Vec::new();
        let scope_temp = TEMPLATES.get_skill_scope_temp_mgr_ref().get_temp(&bt.scope);
        if let Ok(scope_temp) = scope_temp {
            if !scope_temp.scope.is_empty() {
                for direction in scope_temp.scope.iter() {
                    let dir = Direction {
                        direction: &direction.direction,
                    };
                    v.push(dir);
                }
                b.scope = v;
            }
        }
        b
    }
}

pub enum CellType {
    InValid = 0,
    UnUse = 1,
    Valid = 2,
}

///地图
#[derive(Debug, Default, Clone)]
pub struct TileMap {
    pub id: u32,                                   //地图id
    pub map: Vec<Cell>,                            //地图格子vec
    pub coord_map: HashMap<(isize, isize), usize>, //坐标对应格子
    pub world_cell_map: HashMap<u32, u32>,         //世界块map，index，cellid
    pub un_pair_count: i32,                        //未配对地图块数量
}

///块的封装结构体
#[derive(Debug, Default, Clone)]
pub struct Cell {
    pub id: u32,                   //块的配置id
    pub index: usize,              //块的下标
    pub buffs: Vec<Buff>,          //块的效果
    pub is_world: bool,            //是否世界块
    pub element: u8,               //地图块的属性
    pub passive_buffs: Vec<Buff>,  //额外玩家对其添加的buff
    pub user_id: u32,              //这个地图块上面的玩家
    pub pair_index: Option<usize>, //与之配对的下标
    pub x: isize,                  //x轴坐标
    pub y: isize,                  //y轴坐标
}

impl Cell {
    pub fn check_is_locked(&self) -> bool {
        for buff in self.buffs.iter() {
            if buff.id == 321 {
                return true;
            }
        }
        false
    }
}

impl TileMap {
    pub fn get_cell_by_user_id(&self, user_id: u32) -> Option<&Cell> {
        for cell in self.map.iter() {
            if cell.user_id != user_id {
                continue;
            }
            return Some(cell);
        }
        None
    }

    pub fn get_cell_mut_by_user_id(&mut self, user_id: u32) -> Option<&mut Cell> {
        for cell in self.map.iter_mut() {
            if cell.user_id != user_id {
                continue;
            }
            return Some(cell);
        }
        None
    }

    pub fn remove_user(&mut self, user_id: u32) {
        for cell in self.map.iter_mut() {
            if cell.user_id != user_id {
                continue;
            }
            cell.user_id = 0;
            break;
        }
    }

    pub fn get_able_cells(&self) -> Vec<u32> {
        let mut v = Vec::new();
        let tile_map_mgr = crate::TEMPLATES.get_tile_map_temp_mgr_ref();
        let tile_map_temp = tile_map_mgr.temps.get(&4001_u32).unwrap();
        //填充空的格子占位下标
        for index in 0..tile_map_temp.map.len() {
            let res = tile_map_temp.map.get(index).unwrap();
            if *res != 2 {
                continue;
            }
            v.push(index as u32);
        }
        v
    }

    ///初始化战斗地图数据
    pub fn init(member_count: u8, is_open_world_cell: Option<bool>) -> anyhow::Result<Self> {
        let tile_map_mgr = TEMPLATES.get_tile_map_temp_mgr_ref();
        let res = tile_map_mgr.member_temps.get(&member_count);
        if let None = res {
            let str = format!("there is no map config for member_count:{}", member_count);
            anyhow::bail!(str)
        }
        let res = res.unwrap();
        let mut rand = rand::thread_rng();
        let open_world_cell;
        if let Some(res) = is_open_world_cell {
            open_world_cell = res;
        } else {
            let res = rand.gen_range(0, 2);
            open_world_cell = res > 0;
        }

        let res = res.get(&open_world_cell);
        if let None = res {
            let str = format!(
                "there is no map config for member_count:{},is_open_world_cell:{}",
                member_count, open_world_cell
            );
            anyhow::bail!(str)
        }

        let res = res.unwrap();

        let map_random_index = rand.gen_range(0, res.len());
        let tile_map_temp = res.get(map_random_index).unwrap();
        let mut tmd = TileMap::default();
        tmd.id = tile_map_temp.id;
        tmd.map = Vec::with_capacity(30);

        let mut map = [(0, false); 30];
        tile_map_temp.map.iter().enumerate().for_each(|i| {
            let index = i.0;
            let value = i.1;
            let mut cell = Cell::default();
            cell.index = index;
            map[index] = (*value, false);
        });

        let mut empty_v = Vec::new();

        //填充空的格子占位下标
        for index in 0..tile_map_temp.map.len() {
            let res = tile_map_temp.map.get(index).unwrap();
            if *res != 2 {
                continue;
            }
            empty_v.push(index);
        }

        //确定worldcell
        if tile_map_temp.world_cell != 0 {
            let index = rand.gen_range(0, empty_v.len());
            let index_value = empty_v.get(index).unwrap();
            let index_value = *index_value;

            map[index_value] = (tile_map_temp.world_cell, true);
            empty_v.remove(index);
            tmd.world_cell_map
                .insert(index_value as u32, tile_map_temp.world_cell);
        }
        //这里是为了去重，进行拷贝
        let mut random_vec = TEMPLATES.get_cell_temp_mgr_ref().type_vec.clone();
        //然后就是rare_cell
        for cell_rare in tile_map_temp.cell_rare.iter() {
            let type_vec = TEMPLATES
                .get_cell_temp_mgr_ref()
                .rare_map
                .get(&cell_rare.rare)
                .unwrap()
                .clone();
            let mut size = 0;
            'out: loop {
                if size >= cell_rare.count {
                    break 'out;
                }
                for cell_type in type_vec.iter() {
                    if size >= cell_rare.count {
                        break 'out;
                    }
                    //先随出celltype列表中的一个
                    let mut cell_v = hs_2_v(&random_vec.get(cell_type).unwrap());
                    if cell_v.len() == 0 {
                        continue;
                    }
                    let index = rand.gen_range(0, cell_v.len());
                    let cell_id = *cell_v.get(index).unwrap();
                    for _ in 1..=2 {
                        //然后再随机放入地图里
                        let index = rand.gen_range(0, empty_v.len());
                        let index_value = empty_v.get(index).unwrap();
                        map[*index_value] = (cell_id, false);
                        empty_v.remove(index);
                        size += 1;
                    }
                    cell_v.remove(index);
                    //删掉选中的，进行去重
                    random_vec.get_mut(cell_type).unwrap().remove(&cell_id);
                }
            }
        }
        let mut index = 0;
        let mut un_pair_count = 0;
        let mut x = 0_isize;
        let mut y = 0_isize;
        for (cell_id, is_world) in map.iter() {
            if x > 5 {
                x = 0;
                y += 1;
            }

            if y >= 5 {
                y = 0;
            }
            let mut cell = Cell::default();
            cell.id = *cell_id;
            cell.index = index;
            cell.is_world = *is_world;
            cell.x = x;
            cell.y = y;
            tmd.coord_map.insert((x, y), index);
            x += 1;
            let mut buffs: Option<Iter<u32>> = None;
            index += 1;
            if cell.is_world {
                let world_cell = TEMPLATES.get_world_cell_temp_mgr_ref().temps.get(cell_id).unwrap();
                buffs = Some(world_cell.buff.iter());
            } else if cell_id > &(CellType::Valid as u32) {
                let cell_temp = TEMPLATES.get_cell_temp_mgr_ref().temps.get(cell_id).unwrap();
                buffs = Some(cell_temp.buff.iter());
                cell.element = cell_temp.element;
                un_pair_count += 1;
            }
            if let Some(buffs) = buffs {
                let mut buff_array = Vec::new();
                for buff_id in buffs {
                    let buff_temp = crate::TEMPLATES.get_buff_temp_mgr_ref().get_temp(buff_id).unwrap();
                    let buff = Buff::from(buff_temp);
                    buff_array.push(buff);
                }
                cell.buffs = buff_array.clone();
                cell.passive_buffs = buff_array;
            }
            tmd.map.push(cell);
        }
        tmd.un_pair_count = un_pair_count;
        println!("{:?}",map);
        Ok(tmd)
    }
}

fn hs_2_v(hs: &HashSet<u32>) -> Vec<u32> {
    let mut v = Vec::new();
    for i in hs.iter() {
        v.push(*i);
    }
    v
}