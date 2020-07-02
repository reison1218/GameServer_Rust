use std::env;
use std::collections::HashMap;
use tools::templates::tile_map_temp::TileMapTempMgr;
use rand::{random, Rng};
use std::borrow::Borrow;
use tools::templates::template::TemplatesMgr;

pub fn test_map(){
    let path = env::current_dir().unwrap();
    let str = path.as_os_str().to_str().unwrap();
    let res = str.to_string()+"/template";
    let  temp_mgr = tools::templates::template::init_temps(res.as_str());
    let tmd = TileMapData::init(&temp_mgr);
    println!("{:?}",tmd);
}

#[derive(Default,Debug)]
struct TileMapData{
    id:u32,
    map:Vec<u32>,
    world_cell_map:HashMap<u32,u32>,
}

impl TileMapData{
    pub fn init(temp_mgr:&TemplatesMgr)->Self{
        let tile_map_mgr = temp_mgr.get_tile_map_ref();
        let tile_map_temp = tile_map_mgr.temps.get(&4001_u32).unwrap();
        let mut tmd = TileMapData::default();
        tmd.id = 4001_u32;
        let mut map=[0;30];
        let mut index = 0;
        for  i in tile_map_temp.map.iter(){
            map[index] = *i;
            index+=1;
        }
        let mut empty_v = Vec::new();
        //填充空的格子占位下标
        for index in 0..tile_map_temp.map.len(){
            let res = tile_map_temp.map.get(index).unwrap();
            if *res !=2{
                continue;
            }
            empty_v.push(index);
        }
        let mut rand = rand::thread_rng();
        //先随机worldcell
        for cell_id in tile_map_temp.world_cell.iter(){
            if cell_id == &0{
                continue;
            }
            let index = rand.gen_range(0,empty_v.len());
            let index_value = empty_v.get(index).unwrap();
            map[*index_value] = *cell_id;
            empty_v.remove(index);
            tmd.world_cell_map.insert(index as u32,*cell_id);
        }

        //然后决定角色的cell
        let mut cter_id = 1001;
        for i in 1..=tile_map_temp.member_count{
            let cter = temp_mgr.get_character_ref().temps.get(&cter_id).unwrap();
            for j in 1..=cter.cter_cell.count{
                let index = rand.gen_range(0,empty_v.len());
                let index_value = empty_v.get(index).unwrap();
                map[*index_value] = cter.cter_cell.cell_id;
                empty_v.remove(index);
            }
            cter_id+=1;
        }

        //然后就是rare_cell
        for cell_rare in tile_map_temp.cell_rare.iter(){
            let type_vec = temp_mgr.get_cell_ref().rare_map.get(&cell_rare.rare).unwrap();
            println!("rare:{},type_vec:{:?}",cell_rare.rare,type_vec);
            let mut size = 0;
            for cell_type in type_vec.iter(){
                if size >= cell_rare.count{
                    break;
                }

                //先随出celltype列表中的一个
                let cell_v = temp_mgr.get_cell_ref().type_vec.get(cell_type).unwrap();
                let index = rand.gen_range(0,cell_v.len());
                let ss = cell_v.get(index).unwrap();

                for i in 1..=2{
                    //然后再随机放入地图里
                    let index = rand.gen_range(0,empty_v.len());
                    let index_value = empty_v.get(index).unwrap();
                    map[*index_value] = ss.id;
                    empty_v.remove(index);
                    size+=1;
                }
            }
        }
        for i in &map[..]{
            tmd.map.push(*i);
        }
        tmd
    }
}