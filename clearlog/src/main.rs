

use std::fs::File;
use std::time::{Duration,SystemTime};
use chrono::{Local, Date, DateTime, Datelike};
use std::thread;
use std::fs::read_dir;
use std::process::Command;
use std::ops::Index;


const GLOBAL:u64 = 3600000*24;

fn main() {

     loop {
         //执行正常逻辑
         let mut dirs = read_dir("F:/test").unwrap();

         let dt: DateTime<Local> = Local::now();

         //拿到三天前的日期
         let day_time = dt.date().naive_local().with_day(dt.date().day() - 3).unwrap();

         ///删除日期文件
         for dir in dirs {
             let mut d = dir.unwrap();
             let mut str = d.file_name().into_string().unwrap();
             //清空主日志文件内容
            if str.find(".log").is_some(){
                Command::new(">").arg(str.as_str());
            }

             //匹配日期，匹配不上就跳过
             if str.find(day_time.to_string().as_str()).is_none() {
                 continue;
             }
             //组合命令，删除日志文件
             let mut s = String::new();
             s.push_str(" ");
             s.push_str(str.as_str());
             Command::new("rm").arg("-rf").arg(s.as_str());
         }

         //执行完睡1个小时
         let d = Duration::from_millis(GLOBAL);
         thread::sleep(d);
     }
}