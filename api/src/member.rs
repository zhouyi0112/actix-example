use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Result};
use core::{ListFilter, MemberCreate, MemberService};

use crate::server::*;

// init router
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(list);
    cfg.service(create);
}

///
/// http://localhost:8000/member/1/10
/// ?name=aa&phone=1
/// &order=recharge_amount asc
/// &recharge_min=3001&recharge_max=5000
/// &balance_max=10000&balance_min=1000
/// &consume_max=10000&consume_min=1000
///
#[get("/member/{page_size}/{page_num}")]
pub async fn list(
    req: HttpRequest,
    data: web::Data<AppState>,
    page: web::Path<(u64, u64)>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let (page_num, page_size) = page.into_inner();
    // get params
    let filter = web::Query::<ListFilter>::from_query(req.query_string()).unwrap();

    let res = MemberService::list_in_page(conn, page_num, page_size, filter.into_inner()).await;

    Ok(HttpResponse::Ok().json(res.unwrap()))
}

/// http://localhost:8000/member/create
/// ```
/// {
/// "code":"111",
/// "name":"aaa",
/// "phone":"18888888888"
/// }
/// ```
#[post("/member/create")]
pub async fn create(
    data: web::Data<AppState>,
    create_member: web::Json<MemberCreate>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let create_member = create_member.into_inner();

    let res = MemberService::create(conn, create_member).await;

    Ok(HttpResponse::Ok().json(res.unwrap_or(-1)))
}
