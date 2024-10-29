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

/// Simple rate limiter which buckets requests by IP,
/// counts requests and resets the counter every configurable internval.
///
/// Supports `X-Forwarded-For` header if used behind a reverse proxy.
///
/// Response header `X-Rum-Request-Rate` is set to hint the client
/// how frequently they are using the API.
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

    pub fn per_second(limit: u64) -> Self {
        Self::new(Frequency::Second(limit))
    }

    pub fn per_minute(limit: u64) -> Self {
        Self::new(Frequency::Minute(limit))
    }

    pub fn per_hour(limit: u64) -> Self {
        Self::new(Frequency::Hour(limit))
    }

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
