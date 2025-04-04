use serde::Deserialize;
use std::{io, sync::atomic::AtomicU8, time::{Duration, Instant}};
use tokio::{net, sync::mpsc};
use tokio::net::{TcpStream,TcpListener};
use std::collections::VecDeque;


use thiserror::Error;

pub struct Application {
    frontends: Vec<String>,
    backends: VecDeque<String>,
    last_backend: AtomicU8,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("invalid socket address: {0}")]
    InvalidSocket(String),

    #[error("encounterd receiver error")]
    ReciverError,

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("backend not found")]
    BackendNotFound,
}

impl Application {
   pub fn try_from(config: AppConfig) -> Result<Application, AppError> {
        let frontends = config
            .ports
            .iter()
            .map(|&p| {
                 format!("0.0.0.0:{}", p)
            })
            .collect();
        let backends = config.targets;


        Ok(Application {
            frontends,
            backends: backends.into(),
            last_backend: AtomicU8::new(0)
        })
    }

    pub async fn start(&mut self) -> Result<(),AppError> {
        let (request_tx,mut request_rx)= mpsc::unbounded_channel::<TcpStream>();

        start_listeners(self.frontends.clone(),request_tx).await;

        loop {
            let mut  socket = request_rx.recv().await.ok_or(AppError::ReciverError)?;
            let mut backend = self.get_backend().await?;
            tokio::task::spawn(async move {
                match tokio::io::copy_bidirectional(&mut socket, &mut backend).await {
                    Ok(_) => {
                        println!("conn ended successfully");
                    }
                    Err(e) => {
                        println!("conn ended with error: {}",e);
                    }
                }
            });
        }

    }

    pub async fn get_backend(&mut self) -> Result<TcpStream,AppError> {
        let start_time = Instant::now();

        loop {

            if Instant::now().duration_since(start_time) > Duration::from_secs(30) {
                return  Err(AppError::BackendNotFound);
            }
            let backend_addr = self.backends.pop_front().ok_or(AppError::BackendNotFound)?;

            let  backend = match net::TcpStream::connect(backend_addr.clone()).await {
                Ok(b) => b,
                Err(_) => {
                    println!("error connecting backend: {}",&backend_addr);
                    self.backends.push_back(backend_addr);
                    continue;
                },
            };

            println!("using backend: {}",&backend_addr);
            self.backends.push_back(backend_addr);
            return Ok(backend)

        }

    }




}


async fn start_listeners(listening_address: Vec<String>,stream_tx: mpsc::UnboundedSender<TcpStream>)  {
    for address in listening_address {
        let new_sender =stream_tx.clone();
        tokio::spawn(async move {
            listen_on_port(address, new_sender).await
        });
    }

}

async fn listen_on_port(address: String, stream_tx: mpsc::UnboundedSender<TcpStream>) -> Result<tokio::task::JoinHandle<()>,io::Error> {

    let listener =  TcpListener::bind(address.clone()).await?;

    println!("starting listening on address {}",address);

    let handle =     tokio::task::spawn(   async move  {
        loop {
            if let Ok((socket,_)) = listener.accept().await {

            if let Err(_) = stream_tx.send(socket) {
                    println!("receiver dropped");
                    return;
                }
            }
        }
    });
    Ok(handle)
}


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AppConfig {
    name: String,
    ports: Vec<u16>,
    targets: Vec<String>,
}

#[derive(Debug)]
struct Client {
    stream: TcpStream,
}
