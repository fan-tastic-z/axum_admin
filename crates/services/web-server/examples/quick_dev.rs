#![allow(unused)] // For beginning only.

use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
	let hc = httpc_test::new_client("http://localhost:20000")?;

	let req_login = hc.do_post(
		"/api/login",
		json!({
			"username": "demo",
			"password": "demo"
		}),
	);
	req_login.await?.print().await?;

	let req_create_task = hc.do_post(
		"/api/rpc",
		json!({
			"id": 1,
			"method": "create_task",
			"params": {
				"data": {
					"title": "task AAA"
				}
			}
		}),
	);
	let create_task_res = req_create_task.await?;
	create_task_res.print().await?;
	let body = create_task_res.json_body()?;
	let create_task_id = &body["result"]["id"];
	println!("{:?}", create_task_id);

	let req_update_task = hc.do_post(
		"/api/rpc",
		json!({
			"id": 1,
			"method": "update_task",
			"params": {
				"id": create_task_id,
				"data": {
					"title": "task BB"
				}
			}
		}),
	);
	let update_task_res = req_update_task.await?;
	update_task_res.print().await?;

	let req_delete_task = hc.do_post(
		"/api/rpc",
		json!({
			"id": 1,
			"method": "delete_task",
			"params": {
				"id": create_task_id
			}
		}),
	);
	req_delete_task.await?.print().await?;

	let req_list_tasks = hc.do_post(
		"/api/rpc",
		json!({
			"id": 1,
			"method": "list_tasks"
		}),
	);
	req_list_tasks.await?.print().await?;

	let req_logoff = hc.do_post(
		"/api/logoff",
		json!({
			"logoff": true
		}),
	);
	req_logoff.await?.print().await?;
	Ok(())
}
