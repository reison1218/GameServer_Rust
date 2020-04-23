pub mod redistool;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use redis::{
    transaction, Client, Commands, Connection, Pipeline, PipelineCommands, RedisResult, Value,
};
