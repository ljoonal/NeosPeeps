//! NeosVR API implementation.

use chrono::{DateTime, Duration, Local};
use minreq::{Method, Request, Response};
use neos::NeosFriend;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::RwLock;
use std::{fmt::Debug, thread};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NeosLoginResponse {
	pub user_id: String,
	pub token: String,
}

/// The method to interact with the Neos' API
#[derive(Deserialize, Serialize)]
pub struct NeosApi {
	token: String,
	user_id: String,
	machine_id: String,
	#[serde(skip)]
	rate_limited_until: RwLock<Option<chrono::DateTime<chrono::Local>>>,
}

impl Debug for NeosApi {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let redacted = "*****";

		f.debug_struct("NeosApi")
			.field("token", &redacted)
			.field("user_id", &self.user_id)
			.field("machine_id", &redacted)
			.field("rate_limited_until", &self.rate_limited_until)
			.finish()
	}
}

impl NeosApi {
	const API_BASE: &'static str = "https://www.neosvr-api.com/api/";
	/// Creates an API request without any authentication or rate limits.
	fn basic_api_request(method: Method, url: &str) -> Request {
		Request::new(method, &(Self::API_BASE.to_owned() + url))
			.with_header("Accept", "application/json")
			.with_header("Content-Type", "application/json")
			.with_header("User-Agent", crate::USER_AGENT)
	}

	/// Tries make a blocking login API request using an username and a
	/// password.
	pub fn try_login(
		username: &str,
		password: &str,
	) -> Result<Self, &'static str> {
		use rand::distributions::Alphanumeric;
		use rand::{thread_rng, Rng};

		let machine_id: String = thread_rng()
			.sample_iter(&Alphanumeric)
			.take(32)
			.map(char::from)
			.collect();

		let req = Self::basic_api_request(Method::Post, "userSessions");

		let res = req
			.with_json(&json!({
				"username": username,
				"password": password,
				"secretMachineId": &machine_id,
				"rememberMe": true,
			}))
			.map_err(|_| "Failed to send the login request")?
			.send()
			.map_err(|_| "Login request failed")?;

		let json: NeosLoginResponse = match res.json() {
			Ok(v) => v,
			Err(_) => return Err("Login response parsing failed"),
		};

		let api = Self {
			token: json.token,
			user_id: json.user_id,
			machine_id,
			rate_limited_until: Default::default(),
		};

		Ok(api)
	}

	/// Tries make a blocking session extension API request to check session
	/// validity.
	pub fn try_old_session(
		token: String,
		user_id: String,
		machine_id: String,
	) -> Result<Self, &'static str> {
		let api = Self {
			token,
			user_id,
			machine_id,
			rate_limited_until: Default::default(),
		};
		api.extend_session().map_err(|_| "Couldn't validate session")?;

		Ok(api)
	}

	/// Makes the thread sleep until the ratelimit has expired
	fn sleep_if_ratelimited(&self) {
		if let Some(rate_limited_until) =
			*self.rate_limited_until.read().unwrap()
		{
			let millis: u64 = (rate_limited_until.timestamp_millis()
				- Local::now().timestamp_millis())
			.try_into()
			.unwrap_or(1000);
			println!("Neos' API rate limited, sleeping: {}ms", millis);
			thread::sleep(std::time::Duration::from_millis(millis))
		}
	}

	/// Handles status codes and headers that might indicate ratelimit or other
	/// error.
	fn handle_response(&self, res: Response) -> Result<Response, String> {
		let apply_rate_limit = |response: &Response| {
			if let Some(Ok(rate_limit_resets)) = response
				.headers
				.get("X-Rate-Limit-Reset")
				.map(|time| time.parse::<DateTime<Local>>())
			{
				*self.rate_limited_until.write().unwrap() =
					Some(rate_limit_resets);
			} else if let Some(Ok(rate_limit_resets_after)) = response
				.headers
				.get("Retry-After")
				.map(|time| time.parse::<i64>())
			{
				*self.rate_limited_until.write().unwrap() = Some(
					Local::now() + Duration::seconds(rate_limit_resets_after),
				);
			} else {
				*self.rate_limited_until.write().unwrap() =
					Some(Local::now() + Duration::seconds(2));
			}
		};

		if res.status_code == 429 {
			apply_rate_limit(&res);
			return Err("Rate limited".to_owned());
		}

		if let Some(Ok(rate_limit_remaining)) = res
			.headers
			.get("X-Rate-Limit-Remaining")
			.map(|limit| limit.parse::<u32>())
		{
			if rate_limit_remaining == 0 {
				apply_rate_limit(&res);
			}
		}

		if res.status_code < 200 || res.status_code >= 300 {
			return Err("Request wasn't successful (".to_owned()
				+ &res.status_code.to_string()
				+ ")");
		}

		Ok(res)
	}

	fn api_request(
		&self,
		method: Method,
		url: &str,
		build: &mut dyn FnMut(Request) -> Request,
	) -> Result<Response, String> {
		let mut req = Self::basic_api_request(method, url);
		req = req.with_header(
			"Authorization",
			&("neos ".to_owned() + &self.user_id + ":" + &self.token),
		);

		let req = build(req);

		self.sleep_if_ratelimited();

		let res = req.send().map_err(|v| v.to_string())?;

		self.handle_response(res)
	}
	pub fn extend_session(&self) -> Result<(), String> {
		self.api_request(Method::Patch, "userSessions", &mut |r| r)?;
		Ok(())
	}

	pub fn fetch_friends(&self) -> Result<Vec<NeosFriend>, String> {
		let resp = self.api_request(
			Method::Get,
			&("users/".to_owned() + &self.user_id + "/friends"),
			&mut |r| r,
		)?;

		let data: Vec<NeosFriend> = resp.json().map_err(|err| {
			"Couldn't parse friends response (".to_owned()
				+ &err.to_string()
				+ ")"
		})?;

		Ok(data)
	}
}
