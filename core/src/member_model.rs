use sea_orm::{entity::prelude::*, FromQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ListFilter {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub balance_max: Option<Decimal>,
    pub balance_min: Option<Decimal>,
    pub recharge_max: Option<Decimal>,
    pub recharge_min: Option<Decimal>,
    pub consume_max: Option<Decimal>,
    pub consume_min: Option<Decimal>,
    pub order: Option<String>,
}

#[derive(Debug, FromQueryResult, Serialize)]
pub struct MemberListItem {
    pub id: i32,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub balance: Option<Decimal>,
    pub code: String,
    pub registration_time: Option<DateTime>,
    pub level: Option<String>,
    pub salesman_id: Option<i32>,
    pub salesman_name: Option<String>,
    pub recharge_amount: Option<Decimal>,
    pub consume_amount: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct PageResult<T> {
    pub data: Vec<T>,
    pub pages: u64,
    pub page_num: u64,
    pub page_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct MemberCreate {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub code: String,
}
