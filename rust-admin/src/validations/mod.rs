use std::collections::HashMap;
use regex::Regex;
use std::str::FromStr;

lazy_static! { 
    /// 用户名称正则
    static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z]+[a-zA-Z_0-9]{4,19}$").unwrap();
    /// 电子邮件正则
    static ref RE_MAIL: Regex = Regex::new(r"/^([A-Za-z0-9_\-\.])+\@([A-Za-z0-9_\-\.])+\.([A-Za-z]{2,5})$/").unwrap();
}

#[derive(Debug)]
pub struct Validator<'a> { 
    errors: Vec<&'static str>,
    data: &'a HashMap<String, String>
}

pub trait Validation { 
    fn validate(_data: &HashMap<String, String>) -> Result<(), String> { 
        Ok(())
    }
}

impl<'a> Validator<'a> { 
    
    pub fn load(data: &'a HashMap<String, String>) -> Self { 
        Self { 
            errors: vec![],
            data: data,
        }
    }

    /// 是否是用户名称, 6-20位, 英文开头, 数字、下划线、英文
    pub fn is_username(&mut self, field: &'static str, message: &'static str, is_required: bool) -> &mut Self { 
        if let Some(v) = self.data.get(field) {
            let v = v.as_str();
            let count = v.chars().count();
            if count >= 1 && count < 20 && RE_USERNAME.is_match(v) {
                return self;
            }
        }
        if is_required { 
            self.errors.push(message);
        }
        self
    }

    /// 检测是否是正确的密码格式
    pub fn is_password(&mut self, field: &'static str, message: &'static str) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            let count = v.chars().count();
            if count > 5 && count < 20 { 
                return self;
            }
        }
        self.errors.push(message);
        self
    }

    /// 检测是否是正确的验证码
    #[allow(dead_code)]
    pub fn is_check_code(&mut self, field: &'static str, message: &'static str) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            if let Err(_) = v.parse::<usize>() { 
                self.errors.push(message);
            }
        } else { 
            self.errors.push(message);
        }
        self
    }

    /// 判断是否是数字
    pub fn is_numeric(&mut self, field: &'static str, message: &'static str) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            if let Ok(_) = v.parse::<isize>() { 
                return self;
            }
        } 
        self.errors.push(message);
        self
    }

    /// 判断必须是正整数(包括0)
    pub fn is_unsigned(&mut self, field: &'static str, message: &'static str) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            if let Ok(_) = v.parse::<usize>() { 
                return self;
            }
        }
        self.errors.push(message);
        self
    }

    /// 两次输入的内容必须一致
    pub fn equal(&mut self, field: &'static str, equal_field: &'static str, message: &'static str) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            if let Some(e) = self.data.get(equal_field) { 
                if v == e { 
                    return self;
                }
            }
        }
        self.errors.push(message);
        self
    }

    /// 是否是 1/0 的选项
    pub fn is_yes_no(&mut self, field: &'static str, message: &'static str) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            if let Ok(n) = v.parse::<usize>() { 
                if n == 0 || n == 1 { 
                    return self;
                }
            }
        }
        self.errors.push(message);
        self
    }

    /// 是否是 1/0 的选项
    pub fn is_state(&mut self, field: &'static str, message: &'static str) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            if let Ok(n) = v.parse::<usize>() { 
                if n == 0 || n == 1 { 
                    return self;
                }
            }
        }
        self.errors.push(message);
        self
    }

    /// 必须在某个区间范围之内
    pub fn in_range<T: Sized + FromStr + PartialEq>(&mut self, field: &'static str, message: &'static str, array: &[T]) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            if let Ok(n) = v.parse::<T>() { 
                if array.contains(&n) { 
                    return self;
                }
            }
        }
        self.errors.push(message);
        self
    }

    // 判断是否是电子邮件
    //pub fn is_mail(&mut self, field: &'static str, message: &'static str, is_required: bool) -> &mut Self  {
    //    if let Some(v) = self.data.get(field) { 
    //        if RE_MAIL.is_match(v) { 
    //            self.errors.push(message);
    //        }
    //    } else if is_required { 
    //        self.errors.push(message);
    //    }
    //    self
    //}

    /// 指定长度的字符串
    pub fn string_length(&mut self, field: &'static str, message: &'static str, min: usize, max: usize, is_required: bool) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            let word_count = v.chars().count();
            if word_count < min || word_count > max {
                self.errors.push(message);
            }
        } else if is_required { 
            self.errors.push(message);
        }
        self
    }

    /// 限定长度字符串
    pub fn string_limit(&mut self, field: &'static str, message: &'static str, max: usize) -> &mut Self { 
        if let Some(v) = self.data.get(field) { 
            if v.len() > max { 
                self.errors.push(message);
            }
        }
        self
    }

    /// 执行校验
    pub fn validate(&mut self) -> Result<(), String> { 
        if self.errors.len() > 0 { 
            return Err(self.errors.join(","));
        }
        Ok(())
    }
}
