//! Limit how many requests our clients can perform per unit of time.
//!
//! Clients that exceed those limits will have their requests rejected with HTTP `429 - Too Many`.
//! The rate limiting algorithm is nothing fancy: it counts the number of requests, and resets the count
//! every configured amount of time.
//!
//! Clients are bucketed per IP. The rate limiter supports proxies, so if `X-Forwarded-For` header is included, that IP
//! will be used instead. Each response has the `X-Rwf-Request-Rate` header set with the current requests per unit of time,
//! which could help clients self-throttle their request rate.
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use super::{
    super::{Error, Request, Response},
    Middleware, Outcome,
};
use async_trait::async_trait;

#[derive(Default, Debug)]
struct State {
    buckets: HashMap<IpAddr, Counter>,
}

#[derive(Debug)]
struct Counter {
    counter: u64,
    rate: f32,
    last_reset: Instant,
}

impl Counter {
    fn new(last_reset: Instant) -> Self {
        Self {
            counter: 0,
            rate: 0.,
            last_reset,
        }
    }
}

enum Frequency {
    Minute(u64),
    Second(u64),
    Hour(u64),
    Day(u64),
}

impl Frequency {
    pub fn limit(&self) -> u64 {
        use Frequency::*;

        match self {
            Minute(limit) => *limit,
            Second(limit) => *limit,
            Hour(limit) => *limit,
            Day(limit) => *limit,
        }
    }
}

/// Simple rate limiter.
pub struct RateLimiter {
    frequency: Frequency,
    state: Mutex<State>,
}

impl RateLimiter {
    /// New rate limiter with this many requests per second.
    fn new(frequency: Frequency) -> Self {
        Self {
            frequency,
            state: Mutex::new(State::default()),
        }
    }

    /// Create rate limiter with this limit of requests per second.
    pub fn per_second(limit: u64) -> Self {
        Self::new(Frequency::Second(limit))
    }

    /// Create rate limiter with this limit of requests per minute.
    pub fn per_minute(limit: u64) -> Self {
        Self::new(Frequency::Minute(limit))
    }

    /// Create rate limiter with this limit of requests per hour. There is no advanced warning
    /// for clients that reach this limit quickly. If they spend all their requests in the first minute of the hour,
    /// they will be blocked for sending any more for the remainer of the hour.
    pub fn per_hour(limit: u64) -> Self {
        Self::new(Frequency::Hour(limit))
    }

    /// Create rate limiter with this limit of requests per day. There is no advanced warning
    /// for clients that reach this limit quickly. If they spend all their requests in the first hour of the day,
    /// they will be blocked for sending any more for the remainer of the day.
    pub fn per_day(limit: u64) -> Self {
        Self::new(Frequency::Day(limit))
    }
}

#[async_trait]
impl Middleware for RateLimiter {
    async fn handle_request(&self, request: Request) -> Result<Outcome, Error> {
        let peer = match request
            .headers()
            .get("x-forwarded-for")
            .map(|s| crate::peer_addr(s))
        {
            Some(Some(peer)) => peer,
            _ => request.peer().clone(),
        };

        // Get current time before locking mutex.
        // You'd be surprised how slow this function can be.
        let now = Instant::now();

        let reset_duration = match self.frequency {
            Frequency::Second(limit) => Duration::from_millis(1000 * limit),
            Frequency::Minute(limit) => Duration::from_millis(1000 * 60 * limit),
            Frequency::Hour(limit) => Duration::from_millis(1000 * 3600 * limit),
            Frequency::Day(limit) => Duration::from_millis(1000 * 3600 * 24 * limit),
        };

        let too_many = {
            let mut guard = self.state.lock();
            let state = guard
                .buckets
                .entry(peer.ip())
                .or_insert_with(|| Counter::new(now));
            let duration = now.duration_since(state.last_reset);
            state.counter += 1;

            if duration >= reset_duration {
                state.rate = state.counter as f32 / duration.as_secs_f32();
                state.counter = 1;
                state.last_reset = now;
            }

            state.counter > self.frequency.limit()
        };

        if too_many {
            Ok(Outcome::Stop(request, Response::too_many()))
        } else {
            Ok(Outcome::Forward(request))
        }
    }

    async fn handle_response(
        &self,
        request: &Request,
        response: Response,
    ) -> Result<Response, Error> {
        if let Some(rate) = self
            .state
            .lock()
            .buckets
            .get(&request.peer().ip())
            .map(|c| c.rate)
        {
            Ok(response.header("x-rwf-request-rate", rate.to_string()))
        } else {
            Ok(response)
        }
    }
}
