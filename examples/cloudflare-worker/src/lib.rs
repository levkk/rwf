use worker::*;

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    Ok(Response::builder()
        .with_status(200)
        .body(ResponseBody::Empty))
}
