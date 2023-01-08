use anyhow::Result;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

async fn process_socket(socket: TcpStream, server_address: String) -> Result<()> {
    let listener = TcpStream::connect(server_address).await?;
    let (mut reader, mut writer) = listener.into_split();
    let (mut client_reader, mut client_writer) = socket.into_split();

    // If one aborts, the select! statement cancels the other future.
    tokio::select! {
        v = tokio::spawn(async move {
            let mut other_buf = Vec::new();
            loop {
                match reader.read_buf(&mut other_buf).await? {
                    0 => {
                        break;
                    }
                    _ => {}
                }

                client_writer.write_all(&mut other_buf).await?;

                other_buf = Vec::new();
            }

            return Ok(());
        }) => {return v?;}
        v = tokio::spawn(async move {
            let mut buf = Vec::new();
            loop {
                match client_reader.read_buf(&mut buf).await? {
                    0 => {
                        break;
                    }
                    _ => {}
                }

                writer.write_all(&mut buf).await?;

                buf = Vec::new();
            }
            return Ok(());
        }) => {return v?;}
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let server_address = String::from_utf8(std::fs::read("server_addr.txt")?)?;
    let listen_address = "0.0.0.0:25565".parse::<SocketAddr>()?;
    let listener = TcpListener::bind(&listen_address).await?;

    // accept connections and process them
    loop {
        let (socket, _) = match listener.accept().await {
            Ok(n) => n,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let cloned = server_address.clone();

        tokio::spawn(async move {
            match process_socket(socket, cloned).await {
                Err(e) => {
                    eprintln!("{}", e);
                }
                _ => {}
            }
        });
    }
}
