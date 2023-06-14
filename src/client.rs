use devcade_onboard_types::{Request, RequestBody, Response, ResponseBody};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, oneshot, Mutex, OnceCell};

pub struct BackendClient {
  connection: OnceCell<SynchronizedConnection>,
}

type RequestSender = oneshot::Sender<Result<ResponseBody, RequestError>>;
struct SynchronizedConnection {
  requests_tx: mpsc::Sender<(RequestBody, RequestSender)>,
}

#[derive(Debug)]
pub enum RequestError {
  IoError(io::Error),
  ResponseError(String),
  UnexpectedResponse(ResponseBody),
  ChannelClosed,
}

impl fmt::Display for RequestError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::IoError(err) => write!(f, "IoError({err})"),
      Self::ResponseError(err) => write!(f, "ResponseError({err})"),
      Self::UnexpectedResponse(response) => write!(f, "UnexpectedResponse({response})"),
      Self::ChannelClosed => write!(f, "ChannelClosed"),
    }
  }
}

impl From<io::Error> for RequestError {
  fn from(error: io::Error) -> Self {
    Self::IoError(error)
  }
}

impl Default for BackendClient {
  fn default() -> Self {
    Self {
      connection: OnceCell::new(),
    }
  }
}

/// Client for the devcade backend;
/// Allows you to send requests to the backend and get their responses.
///
/// This struct represents an underlying connection to the devcade backend, so
/// try not to make more than one.
///
/// # Example
/// ```
/// let backend_client: BackendClient = Default::default();
/// let pong = backend_client.send(RequestBody::Ping).await.unwrap();
/// println!("Pong! {pong}");
/// ```
impl BackendClient {
  async fn create_connection() -> Result<SynchronizedConnection, io::Error> {
    let (connection_reader, mut connection_writer) = UnixStream::connect(
      std::env::var("DEVCADE_ONBOARD_PATH").unwrap_or("/tmp/devcade/onboard.sock".to_owned()),
    )
    .await?
    .into_split();
    let (requests_tx, mut requests_rx) = mpsc::channel::<(RequestBody, RequestSender)>(100);
    let listeners = Arc::new(Mutex::new(HashMap::<u32, RequestSender>::new()));
    {
      let listeners = listeners.clone();
      tokio::spawn(async move {
        let mut request_id_counter = 0;
        while let Some((body, callback_tx)) = requests_rx.recv().await {
          let mut listeners = listeners.lock().await;
          while listeners.contains_key(&request_id_counter) {
            request_id_counter = request_id_counter.wrapping_add(1);
          }
          let request_id = request_id_counter;
          let request = Request { request_id, body };

          let mut frame = serde_json::to_vec(&request).expect("Couldn't serialize RequestBody?");
          frame.push(b'\n');
          if let Err(err) = connection_writer.write_all(&frame).await {
            if let Err(Err(err)) = callback_tx.send(Err(err.into())) {
              log::error!("Couldn't send message to callback! Message we were asked to send was: {request:?}. Failed because {err}");
            }
            return;
          }
          listeners.insert(request_id, callback_tx);
        }
      });
    }
    tokio::spawn(async move {
      let connection_reader = BufReader::new(connection_reader);
      let mut lines = connection_reader.lines();
      while let Ok(Some(line)) = lines.next_line().await {
        let response: Response = match serde_json::from_str(&line) {
          Ok(response) => response,
          Err(err) => {
            log::error!("Couldn't decode response ({line}) {err}");
            continue;
          }
        };

        let request_id = &response.request_id;
        let mut listeners = listeners.lock().await;
        let handler = match listeners.remove(request_id) {
          Some(handler) => handler,
          None => {
            log::error!(
              "Got response for request ID {request_id} that we weren't expecting! {response}"
            );
            continue;
          }
        };
        std::mem::drop(listeners);

        if handler
          .send(match response.body {
            ResponseBody::Err(err) => Err(RequestError::ResponseError(err)),
            body => Ok(body),
          })
          .is_err()
        {
          log::error!("Failed to send response for {request_id} because the other side of the callback closed");
        }
      }
    });
    Ok(SynchronizedConnection { requests_tx })
  }

  async fn get_connection(&self) -> Result<&SynchronizedConnection, io::Error> {
    self
      .connection
      .get_or_try_init(Self::create_connection)
      .await
  }

  /// Sends a request to the backend and returns the corresponding response.
  /// If the response is [`ResponseBody::Err`],
  /// a [`RequestError::ResponseError`] is returned instead with the error
  /// message.
  pub async fn send(&self, body: RequestBody) -> Result<ResponseBody, RequestError> {
    let connection = self.get_connection().await?;
    let (tx, rx) = oneshot::channel();
    connection
      .requests_tx
      .send((body, tx))
      .await
      .map_err(|_| RequestError::ChannelClosed)?;
    match rx.await.map_err(|_| RequestError::ChannelClosed) {
      Ok(Ok(response)) => Ok(response),
      Ok(Err(err)) | Err(err) => Err(err),
    }
  }
}
