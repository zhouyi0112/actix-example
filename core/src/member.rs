use ::entity::{member, member::Entity as Member};
use chrono::Local;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, DbBackend, Set, Statement};

use crate::member_model::*;

pub struct MemberService;

impl MemberService {
    /// generate like string
    pub fn like_str(keyword: Option<String>) -> Value {
        let v = format!("%{}%", keyword.unwrap_or_default());
        Value::String(Some(Box::new(v)))
    }

    /// member page list
    pub async fn list_in_page(
        db: &DbConn,
        page_num: u64,
        page_size: u64,
        filter: ListFilter,
    ) -> Result<PageResult<MemberListItem>, DbErr> {
        let mut sql = r#"
        SELECT
            M.*,
            E.name AS  "salesman_name",
            SUM(CR.recharge_amount) AS "recharge_amount",
            SUM(CR.consume_amount) AS "consume_amount"
        FROM
            "member" M
        LEFT JOIN "employee" as E ON M.salesman_id = E.id
        LEFT JOIN "consume_record" AS CR ON M.id = CR.member_id
        "#
        .to_string();

        let mut values: Vec<Value> = vec![];
        let mut index = 1;

        // WHERE
        if filter.name.is_some()
            || filter.phone.is_some()
            || filter.balance_max.is_some()
            || filter.balance_min.is_some()
        {
            sql.push_str(" WHERE ");
            let mut where_and_flag = false;

            if filter.name.is_some() {
                where_and_flag = true;
                sql.push_str(format!("M.name LIKE ${}", index).as_str());
                index += 1;
                values.push(Self::like_str(filter.name));
            }

            if filter.phone.is_some() {
                if where_and_flag {
                    sql.push_str(" AND ");
                } else {
                    where_and_flag = true;
                }
                sql.push_str(format!("M.phone LIKE ${}", index).as_str());
                index += 1;
                values.push(Self::like_str(filter.phone));
            }

            if filter.balance_max.is_some() {
                if where_and_flag {
                    sql.push_str(" AND ");
                } else {
                    where_and_flag = true;
                }
                sql.push_str(format!("M.balance <= ${}", index).as_str());
                index += 1;
                values.push(filter.balance_max.into());
            }

            if filter.balance_min.is_some() {
                if where_and_flag {
                    sql.push_str(" AND ");
                } else {
                    where_and_flag = true;
                }
                sql.push_str(format!("M.balance >= ${}", index).as_str());
                index += 1;
                values.push(filter.balance_min.into());
            }
        }

        // GROUP
        sql.push_str(" GROUP BY ");
        sql.push_str("M.id, E.name");

        // HAVING
        if filter.recharge_max.is_some()
            || filter.recharge_min.is_some()
            || filter.consume_max.is_some()
            || filter.consume_min.is_some()
        {
            sql.push_str(" HAVING ");
            let mut having_and_flag = false;

            if filter.recharge_max.is_some() {
                sql.push_str(format!("SUM(CR.recharge_amount) <= ${} ", index).as_str());
                index += 1;
                values.push(filter.recharge_max.into());
                having_and_flag = true;
            }

            if filter.recharge_min.is_some() {
                if having_and_flag {
                    sql.push_str(" AND ");
                } else {
                    having_and_flag = true;
                }
                sql.push_str(format!("SUM(CR.recharge_amount) >= ${} ", index).as_str());
                index += 1;
                values.push(filter.recharge_min.into());
            }

            if filter.consume_max.is_some() {
                if having_and_flag {
                    sql.push_str(" AND ");
                } else {
                    having_and_flag = true;
                }
                sql.push_str(format!("SUM(CR.consume_amount) <= ${} ", index).as_str());
                index += 1;
                values.push(filter.consume_max.into());
            }

            if filter.consume_min.is_some() {
                if having_and_flag {
                    sql.push_str(" AND ");
                } else {
                    having_and_flag = true;
                }
                sql.push_str(format!("SUM(CR.consume_amount) >= ${} ", index).as_str());
                index += 1;
                values.push(filter.consume_min.into());
            }
        }

        if filter.order.is_some() {
            sql.push_str(format!(" ORDER BY {}", filter.order.unwrap()).as_str());
        }

        let paginator = Member::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                sql.as_str(),
                values,
            ))
            .into_model::<MemberListItem>()
            .paginate(db, page_size);

        // get pages
        let num_pages = paginator.num_pages().await;
        // err or 0 pages
        let pages = num_pages.unwrap_or(0);
        if pages <= 0 {
            return Ok(PageResult {
                page_num: page_num,
                page_size: page_size,
                data: vec![],
                pages: 0,
            });
        }

        // fetch data
        let data = paginator.fetch_page(page_num - 1).await;

        Ok(PageResult {
            page_num,
            page_size: page_size,
            data: data.unwrap_or_default(),
            pages: pages,
        })
    }

    /// create new member
    pub async fn create(db: &DbConn, create_member: MemberCreate) -> Result<i32, DbErr> {
        let res = member::ActiveModel {
            id: NotSet,
            name: Set(create_member.name.to_owned()),
            phone: Set(create_member.phone.to_owned()),
            code: Set(create_member.code.to_owned()),
            // init
            balance: Set(Decimal::ZERO),
            registration_time: Set(Local::now().naive_local()),
            create_time: Set(Local::now().naive_local()),
            update_time: Set(None),
            level: Set(None),
            salesman_id: Set(None),
            ..Default::default()
        }
        .save(db)
        .await;

        if res.is_ok() {
            return Ok(res.unwrap().id.unwrap());
        }

        Ok(-1)
    }
}
