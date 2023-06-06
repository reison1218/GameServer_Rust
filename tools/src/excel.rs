use std::collections::HashMap;

use calamine::{open_workbook, DataType, Error, Reader, Xlsx};

pub fn read(path: &str) -> Result<Vec<HashMap<String, DataType>>, Error> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or(Error::Msg("Cannot find sheet"))??;
    let mut res = Vec::new();

    let mut range_index = 0;

    let mut keys = HashMap::new();
    let mut none_index = 0;
    for data_type in range.rows() {
        range_index += 1;
        if range_index == 2 {
            for i in 0..data_type.len() {
                let cell = &data_type[i];
                if cell.is_empty() && none_index == 0 {
                    none_index = i;
                    continue;
                }
                if none_index != 0 && i >= none_index {
                    continue;
                }
                keys.insert(i, cell.as_string().unwrap());
            }
        }
        if range_index > 4 {
            let mut map = HashMap::new();
            for i in 0..data_type.len() {
                if none_index != 0 && i >= none_index {
                    continue;
                }
                let key = keys.get(&i).unwrap();
                map.insert(key.clone(), data_type[i].clone());
            }
            res.push(map);
        }
    }
    Ok(res)
}

pub fn read_disc(path: &str) -> Result<Vec<HashMap<String, DataType>>, Error> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or(Error::Msg("Cannot find sheet"))??;
    let mut res = Vec::new();

    let mut range_index = 0;

    for data_type in range.rows() {
        range_index += 1;
        if range_index > 4 {
            let key_cell = data_type[0].clone();
            let value_cell = data_type[2].clone();
            let mut map = HashMap::new();
            map.insert(key_cell.as_string().unwrap(), value_cell);
            res.push(map);
        }
    }
    Ok(res)
}
