use corro_admin::{Command, LogLevel, Response};
use eyre::eyre;
use futures::{SinkExt, TryStreamExt};
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(windows)]
use tokio::net::TcpStream;
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::LengthDelimitedCodec;
use tracing::{error, event};

#[cfg(unix)]
type FramedStream = Framed<
    tokio_util::codec::Framed<UnixStream, LengthDelimitedCodec>,
    Response,
    Command,
    Json<Response, Command>,
>;

#[cfg(windows)]
type FramedStream = Framed<
    tokio_util::codec::Framed<TcpStream, LengthDelimitedCodec>,
    Response,
    Command,
    Json<Response, Command>,
>;

pub struct AdminConn {
    stream: FramedStream,
}

impl AdminConn {
    #[cfg(unix)]
    pub async fn connect<P: AsRef<Path>>(path: P) -> eyre::Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Self::from_stream(stream)
    }

    #[cfg(windows)]
    pub async fn connect_tcp(port: u16) -> eyre::Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
        Self::from_stream(stream)
    }

    #[cfg(unix)]
    fn from_stream(stream: UnixStream) -> eyre::Result<Self> {
        Ok(Self {
            stream: Framed::new(
                tokio_util::codec::Framed::new(
                    stream,
                    LengthDelimitedCodec::builder()
                        .max_frame_length(100 * 1_024 * 1_024)
                        .new_codec(),
                ),
                Json::default(),
            ),
        })
    }

    #[cfg(windows)]
    fn from_stream(stream: TcpStream) -> eyre::Result<Self> {
        Ok(Self {
            stream: Framed::new(
                tokio_util::codec::Framed::new(
                    stream,
                    LengthDelimitedCodec::builder()
                        .max_frame_length(100 * 1_024 * 1_024)
                        .new_codec(),
                ),
                Json::default(),
            ),
        })
    }

    pub async fn send_command(&mut self, cmd: Command) -> eyre::Result<()> {
        self.stream.send(cmd).await?;

        loop {
            let res = self.stream.try_next().await?;

            match res {
                None => {
                    error!("Failed to get response from Corrosion's admin!");
                    return Err(eyre!("Failed to get response from Corrosion's admin!"));
                }
                Some(res) => match res {
                    Response::Log { level, msg, ts } => match level {
                        LogLevel::Trace => event!(tracing::Level::TRACE, %ts, "{msg}"),
                        LogLevel::Debug => event!(tracing::Level::DEBUG, %ts, "{msg}"),
                        LogLevel::Info => event!(tracing::Level::INFO, %ts, "{msg}"),
                        LogLevel::Warn => event!(tracing::Level::WARN, %ts, "{msg}"),
                        LogLevel::Error => event!(tracing::Level::ERROR, %ts, "{msg}"),
                    },
                    Response::Error { msg } => {
                        error!("{msg}");
                        return Err(eyre!("{msg}"));
                    }
                    Response::Success => {
                        break;
                    }
                    Response::Json(json) => {
                        println!("{}", serde_json::to_string_pretty(&json).unwrap());
                    }
                },
            }
        }

        Ok(())
    }
}
