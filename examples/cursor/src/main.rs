use rwf::controller::TurboStream;
use rwf::model::migrate;
use rwf::model::pool::Transaction;
use rwf::model::prelude::*;
use rwf::prelude::*;
use std::collections::BTreeMap;
use std::str::FromStr;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, macros::Model, PartialEq, Eq)]
pub struct AppLog {
    id: Option<i64>,
    ts: OffsetDateTime,
    data: String,
}
impl AppLog {
    pub fn q() -> Scope<Self> {
        Self::all().order(("id", "asc"))
    }
    pub async fn cursor(name: impl ToString, tx: &mut Transaction) -> ModelCursor<Self> {
        Self::q()
            .declare_cursor(name)
            .expect("Static Query is correct")
            .scroll()
            .hold()
            .create_model_cursor(tx)
            .await
            .expect("Database is able to serve the request")
    }
}

#[derive(Debug, Default)]
struct TxModelController {
    cursors: Mutex<BTreeMap<SessionId, ModelCursor<AppLog>>>,
}
enum Direction {
    Next,
    Prev,
}
impl FromStr for Direction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "next" => Ok(Self::Next),
            "prev" => Ok(Self::Prev),
            s => Err(format!("{} is not a valid direction", s)),
        }
    }
}
static FETCH_CHUNK: i64 = 3;
#[async_trait]
impl Controller for TxModelController {
    async fn handle(&self, request: &Request) -> Result<Response, rwf::controller::Error> {
        let mut tx = Pool::begin().await?;
        let sess_id = request.session_id();
        let cur_name = format!("sess_cur_{}", sess_id.to_string());
        match request.query().get::<Direction>("direction") {
            None => {
                if let Some(old) = self.cursors.lock().await.remove(&sess_id) {
                    tx.query_cached(old.close_stmt().as_str(), &[]).await?;
                }
                self.cursors
                    .lock()
                    .await
                    .insert(sess_id, AppLog::cursor(cur_name, &mut tx).await);
                tx.commit().await?;
                render!(request, "templates/index.html")
            }
            Some(direction) => {
                let mut cur = match self.cursors.lock().await.remove(&sess_id) {
                    None => return Ok(Response::new().redirect(request.path().path())),
                    Some(cursor) => cursor,
                }
                .to_transaction_cursor(tx);
                let entries = match direction {
                    Direction::Next => cur.fetch_optional(cur.fetch_stmt().forward(FETCH_CHUNK)),
                    Direction::Prev => cur.fetch_optional(cur.fetch_stmt().backward(FETCH_CHUNK)),
                }
                .await?
                .unwrap_or(Vec::new());
                let (cur, tx) = cur.decouple();
                tx.commit().await?;
                self.cursors.lock().await.insert(sess_id, cur);
                Ok(Response::new().turbo_stream(&[turbo_stream!(request, "templates/entries.html", "entries", "entries" => entries).action("replace")]))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), rwf::http::Error> {
    migrate().await?;
    rwf::http::server::Server::new(vec![
        route!("/" => TxModelController),
        route!("/ws" => TurboStream),
    ])
    .launch()
    .await
}

#[cfg(test)]
mod tests {
    use super::AppLog;
    use rwf::model::migrate;
    use rwf::model::prelude::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_cur_eq_query() {
        migrate().await.unwrap();
        let query = AppLog::all().order(("id", "asc"));
        let mut tx = Pool::pool().transaction().await.unwrap();
        let query_res = query.clone().fetch_all(&mut tx).await.unwrap();
        let mut cur = query
            .declare_cursor("cursor")
            .unwrap()
            .create_tx_model_cursor(Some(tx))
            .await
            .unwrap();
        let cur_res: Vec<AppLog> = cur
            .stream(cur.fetch_stmt())
            .map(|log| log.unwrap())
            .collect()
            .await;
        cur.close().await.unwrap();
        assert_eq!(query_res, cur_res);
    }
}
