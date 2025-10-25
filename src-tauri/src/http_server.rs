use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use warp::Filter;

#[derive(Debug, Clone)]
pub struct CallbackResult {
    pub code: String,
    pub state: String,
}

pub struct HttpServer {
    shutdown_tx: Option<oneshot::Sender<()>>,
    result_receiver: Arc<Mutex<Option<oneshot::Receiver<Result<CallbackResult, String>>>>>,
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            shutdown_tx: None,
            result_receiver: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start_and_wait_for_callback(&mut self) -> Result<CallbackResult, String> {
        let (result_tx, result_rx) = oneshot::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        
        self.shutdown_tx = Some(shutdown_tx);
        *self.result_receiver.lock().unwrap() = Some(result_rx);

        let result_tx = Arc::new(Mutex::new(Some(result_tx)));

        // Create the callback route
        let callback_route = warp::path!("oauth" / "callback")
            .and(warp::query::<HashMap<String, String>>())
            .and_then({
                let result_tx = result_tx.clone();
                move |params: HashMap<String, String>| {
                    let result_tx = result_tx.clone();
                    async move {
                        let code = params.get("code");
                        let state = params.get("state");
                        let error = params.get("error");

                        if let Some(error) = error {
                            let default_error = "Unknown error".to_string();
                            let error_description = params.get("error_description")
                                .unwrap_or(&default_error);
                            let error_msg = format!("OAuth error: {} - {}", error, error_description);
                            
                            if let Some(tx) = result_tx.lock().unwrap().take() {
                                let _ = tx.send(Err(error_msg));
                            }
                            
                            return Ok::<warp::reply::Html<&str>, warp::Rejection>(warp::reply::html(
                                "<html><body><h1>登录失败</h1><p>OAuth授权失败，请关闭此窗口并重试。</p></body></html>"
                            ));
                        }

                        match (code, state) {
                            (Some(code), Some(state)) => {
                                let callback_result = CallbackResult {
                                    code: code.clone(),
                                    state: state.clone(),
                                };

                                if let Some(tx) = result_tx.lock().unwrap().take() {
                                    let _ = tx.send(Ok(callback_result));
                                }

                                Ok::<warp::reply::Html<&str>, warp::Rejection>(warp::reply::html(
                                    "<html><body><h1>登录成功</h1><p>授权成功！请关闭此窗口返回应用。</p></body></html>"
                                ))
                            }
                            _ => {
                                let error_msg = "Missing code or state parameter".to_string();
                                if let Some(tx) = result_tx.lock().unwrap().take() {
                                    let _ = tx.send(Err(error_msg));
                                }

                                Ok::<warp::reply::Html<&str>, warp::Rejection>(warp::reply::html(
                                    "<html><body><h1>登录失败</h1><p>缺少必要参数，请关闭此窗口并重试。</p></body></html>"
                                ))
                            }
                        }
                    }
                }
            });

        let routes = callback_route;

        let (addr, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(([127, 0, 0, 1], 8765), async {
                shutdown_rx.await.ok();
            });

        println!("OAuth callback server started on http://{}", addr);

        // Start the server in a background task
        tokio::spawn(server);

        // Wait for the callback result
        let result_rx = self.result_receiver.lock().unwrap().take()
            .ok_or("No result receiver available")?;

        match result_rx.await {
            Ok(result) => result,
            Err(_) => Err("Failed to receive callback result".to_string()),
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for HttpServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}
