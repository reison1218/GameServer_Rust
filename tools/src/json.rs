pub type JsonValue = serde_json::Value;

pub trait JsonValueTrait {
    fn get_bool(&self, key: &str) -> Option<bool>;
    fn get_u8(&self, key: &str) -> Option<u8>;
    fn get_i8(&self, key: &str) -> Option<i8>;
    fn get_u16(&self, key: &str) -> Option<u16>;
    fn get_i16(&self, key: &str) -> Option<i16>;
    fn get_u32(&self, key: &str) -> Option<u32>;
    fn get_i32(&self, key: &str) -> Option<i32>;
    fn get_i64(&self, key: &str) -> Option<i64>;
    fn get_u64(&self, key: &str) -> Option<u64>;
    fn get_usize(&self, key: &str) -> Option<usize>;
    fn get_isize(&self, key: &str) -> Option<isize>;
    fn get_str(&self, key: &str) -> Option<&str>;
    fn get_object(&self, key: &str) -> Option<&serde_json::Map<String, JsonValue>>;
    fn get_object_mut(&mut self, key: &str) -> Option<&mut serde_json::Map<String, JsonValue>>;
    fn get_array(&self, key: &str) -> Option<&Vec<JsonValue>>;
    fn get_array_mut(&mut self, key: &str) -> Option<&mut Vec<JsonValue>>;
    fn get_null(&self, key: &str) -> Option<()>;
}

impl JsonValueTrait for JsonValue {
    fn get_bool(&self, key: &str) -> Option<bool> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_bool();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }
    fn get_u8(&self, key: &str) -> Option<u8> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as u8)
    }
    fn get_i8(&self, key: &str) -> Option<i8> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as i8)
    }
    fn get_u16(&self, key: &str) -> Option<u16> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as u16)
    }
    fn get_i16(&self, key: &str) -> Option<i16> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as i16)
    }
    fn get_u32(&self, key: &str) -> Option<u32> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as u32)
    }
    fn get_i32(&self, key: &str) -> Option<i32> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as i32)
    }
    fn get_i64(&self, key: &str) -> Option<i64> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }
    fn get_u64(&self, key: &str) -> Option<u64> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }
    fn get_usize(&self, key: &str) -> Option<usize> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as usize)
    }

    fn get_isize(&self, key: &str) -> Option<isize> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as isize)
    }

    fn get_str(&self, key: &str) -> Option<&str> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_str();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }
    fn get_object(&self, key: &str) -> Option<&serde_json::Map<String, JsonValue>> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_object();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_object_mut(&mut self, key: &str) -> Option<&mut serde_json::Map<String, JsonValue>> {
        let res = self.get_mut(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_object_mut();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_array(&self, key: &str) -> Option<&Vec<JsonValue>> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_array();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_array_mut(&mut self, key: &str) -> Option<&mut Vec<JsonValue>> {
        let res = self.get_mut(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_array_mut();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_null(&self, key: &str) -> Option<()> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.as_null();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }
}

impl JsonValueTrait for serde_json::Map<String, JsonValue> {
    fn get_bool(&self, key: &str) -> Option<bool> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_bool();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_u8(&self, key: &str) -> Option<u8> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as u8)
    }

    fn get_i8(&self, key: &str) -> Option<i8> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as i8)
    }

    fn get_u16(&self, key: &str) -> Option<u16> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as u16)
    }

    fn get_i16(&self, key: &str) -> Option<i16> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as i16)
    }

    fn get_u32(&self, key: &str) -> Option<u32> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as u32)
    }

    fn get_i32(&self, key: &str) -> Option<i32> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as i32)
    }

    fn get_i64(&self, key: &str) -> Option<i64> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as i64)
    }

    fn get_u64(&self, key: &str) -> Option<u64> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_usize(&self, key: &str) -> Option<usize> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_u64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as usize)
    }

    fn get_isize(&self, key: &str) -> Option<isize> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_i64();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap() as isize)
    }

    fn get_str(&self, key: &str) -> Option<&str> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_str();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_object(&self, key: &str) -> Option<&serde_json::Map<String, JsonValue>> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_object();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_object_mut(&mut self, key: &str) -> Option<&mut serde_json::Map<String, JsonValue>> {
        let res = self.get_mut(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_object_mut();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_array(&self, key: &str) -> Option<&Vec<JsonValue>> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_array();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_array_mut(&mut self, key: &str) -> Option<&mut Vec<JsonValue>> {
        let res = self.get_mut(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_array_mut();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }

    fn get_null(&self, key: &str) -> Option<()> {
        let res = self.get(key);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap().as_null();
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }
}
