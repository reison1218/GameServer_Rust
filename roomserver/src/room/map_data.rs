use crate::battle::battle_buff::Buff;
use crate::battle::battle_enum::buff_type::LOCKED;
use crate::TEMPLATES;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::slice::Iter;

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum CellType {
    InValid = 0,
    UnUse = 1,
    Valid = 2,
}

///地图
#[derive(Debug, Default, Clone)]
pub struct TileMap {
    pub id: u32,                                   //地图id
    pub map: [Cell; 30],                           //地图格子vec
    pub coord_map: HashMap<(isize, isize), usize>, //坐标对应格子
    pub world_cell_map: HashMap<u32, u32>,         //世界块map，index，cellid
    pub un_pair_count: i32,                        //未配对地图块数量
}

///块的封装结构体
#[derive(Debug, Default, Clone)]
pub struct Cell {
    pub id: u32,                   //块的配置id
    pub index: usize,              //块的下标
    pub buffs: HashMap<u32, Buff>, //块的效果
    pub is_world: bool,            //是否世界块
    pub element: u8,               //地图块的属性
    pub passive_buffs: Vec<u32>,   //额外玩家对其添加的buff
    pub user_id: u32,              //这个地图块上面的玩家
    pub pair_index: Option<usize>, //与之配对的下标
    pub x: isize,                  //x轴坐标
    pub y: isize,                  //y轴坐标
}

impl Cell {
    pub fn check_is_locked(&self) -> bool {
        for buff in self.buffs.values() {
            if buff.id == LOCKED {
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
        let tile_map_mgr = crate::TEMPLATES.get_tile_map_ref();
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
    pub fn init(member_count: u32, is_open_world_cell: Option<bool>) -> anyhow::Result<Self> {
        let tile_map_mgr = TEMPLATES.get_tile_map_ref();
        let res = tile_map_mgr.member_temps.get(&member_count);
        if let None = res {
            let str = format!("there is no map config for member_count:{}", member_count);
            anyhow::bail!(str)
        }
        let res = res.unwrap();
        let mut rand = rand::thread_rng();
        let open_world_cell;
        //如果开启世界块，则直接拿到有世界块的状态
        if let Some(res) = is_open_world_cell {
            open_world_cell = res;
        } else {
            //否则进行随机，0-1，0代表不开启世界块
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

        let tile_map_temp_v = res.unwrap();

        let map_random_index = rand.gen_range(0, tile_map_temp_v.len());
        let tile_map_temp = tile_map_temp_v.get(map_random_index).unwrap();
        let mut tmp = TileMap::default();
        tmp.id = tile_map_temp.id;

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
            tmp.world_cell_map
                .insert(index_value as u32, tile_map_temp.world_cell);
        }
        //这里是为了去重，进行拷贝
        let mut random_vec = TEMPLATES.get_cell_ref().type_vec.clone();
        //然后就是rare_cell
        for cell_rare in tile_map_temp.cell_rare.iter() {
            let type_vec = TEMPLATES
                .get_cell_ref()
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
        let mut v_v = Vec::new();
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
            tmp.coord_map.insert((x, y), index);
            x += 1;
            let mut buffs: Option<Iter<u32>> = None;
            if cell.is_world {
                let world_cell = TEMPLATES.get_world_cell_ref().temps.get(cell_id).unwrap();
                buffs = Some(world_cell.buff.iter());
            } else if cell_id > &(CellType::Valid as u32) {
                let cell_temp = TEMPLATES.get_cell_ref().temps.get(cell_id).unwrap();
                buffs = Some(cell_temp.buff.iter());
                cell.element = cell_temp.element;
                un_pair_count += 1;
            }
            if let Some(buffs) = buffs {
                let mut buff_map = HashMap::new();
                for buff_id in buffs {
                    let buff_temp = crate::TEMPLATES.get_buff_ref().get_temp(buff_id).unwrap();
                    let buff = Buff::from(buff_temp);
                    buff_map.insert(buff.id, buff);
                    cell.passive_buffs.push(*buff_id);
                }
                cell.buffs = buff_map;
            }
            tmp.map[index] = cell;
            v_v.push((index, *cell_id));
            index += 1;
        }
        println!("{:?}", v_v);
        tmp.un_pair_count = un_pair_count;
        Ok(tmp)
    }
}

fn hs_2_v(hs: &HashSet<u32>) -> Vec<u32> {
    let mut v = Vec::new();
    for i in hs.iter() {
        v.push(*i);
    }
    v
}
