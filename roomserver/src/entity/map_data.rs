use rand::Rng;
use std::collections::{HashMap, HashSet};
use tools::templates::template::TemplatesMgr;
use tools::templates::tile_map_temp::TileMapTemp;

///地图
#[derive(Debug, Default, Clone)]
pub struct TileMap {
    pub id: u32,                           //地图id
    pub map: Vec<u32>,                     //地图格子vec
    pub world_cell_map: HashMap<u32, u32>, //世界块map，index，cellid
}

impl TileMap {
    #[warn(dead_code)]
    pub fn new(temp: &TileMapTemp) -> anyhow::Result<Self> {
        let id = temp.id;
        let map = temp.map.clone();
        let mut cell_array = Vec::new();
        for v in map {
            cell_array.push(v);
        }
        Ok(TileMap {
            id,
            map: cell_array,
            world_cell_map: HashMap::new(),
        })
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

    pub fn init(temp_mgr: &TemplatesMgr, cters: Vec<u32>) -> Self {
        let tile_map_mgr = temp_mgr.get_tile_map_ref();
        let tile_map_temp = tile_map_mgr.temps.get(&4001_u32).unwrap();
        let mut tmd = TileMap::default();
        tmd.id = 4001_u32;
        let mut map = [0; 30];
        let mut index = 0;
        for i in tile_map_temp.map.iter() {
            map[index] = *i;
            index += 1;
        }
        let mut empty_v = Vec::new();
        //填充空的格子占位下标
        for index in 0..tile_map_temp.map.len() {
            let res = tile_map_temp.map.get(index).unwrap();
            if *res != 2 {
                continue;
            }
            empty_v.push(index);
        }
        let mut rand = rand::thread_rng();
        //先随机worldcell
        for cell_id in tile_map_temp.world_cell.iter() {
            if cell_id == &0 {
                continue;
            }
            let index = rand.gen_range(0, empty_v.len());
            let index_value = empty_v.get(index).unwrap();
            let index_value = *index_value;
            map[index_value] = *cell_id;
            empty_v.remove(index);
            tmd.world_cell_map.insert(index_value as u32, *cell_id);
        }

        //然后决定角色的cell
        for cter_id in cters {
            let cter = temp_mgr.get_character_ref().temps.get(&cter_id).unwrap();
            for _ in 1..=cter.cter_cell.count {
                let index = rand.gen_range(0, empty_v.len());
                let index_value = empty_v.get(index).unwrap();
                map[*index_value] = cter.cter_cell.cell_id;
                empty_v.remove(index);
            }
        }

        //然后就是rare_cell
        for cell_rare in tile_map_temp.cell_rare.iter() {
            let type_vec = temp_mgr
                .get_cell_ref()
                .rare_map
                .get(&cell_rare.rare)
                .unwrap();
            let mut size = 0;
            for cell_type in type_vec.iter() {
                if size >= cell_rare.count {
                    break;
                }

                //先随出celltype列表中的一个
                let cell_v = hs_2_v(&temp_mgr.get_cell_ref().type_vec.get(cell_type).unwrap());
                let index = rand.gen_range(0, cell_v.len());
                let ss = cell_v.get(index).unwrap();

                for _ in 1..=2 {
                    //然后再随机放入地图里
                    let index = rand.gen_range(0, empty_v.len());
                    let index_value = empty_v.get(index).unwrap();
                    map[*index_value] = *ss;
                    empty_v.remove(index);
                    size += 1;
                }
            }
        }
        for i in &map[..] {
            tmd.map.push(*i);
        }
        tmd
    }
}

fn hs_2_v(hs: &HashSet<u32>) -> Vec<u32> {
    let mut v = Vec::new();
    for i in hs.iter() {
        v.push(*i);
    }
    v
}
