use std::net::Shutdown;

use async_std::{
    io::{self, prelude::BufReadExt, ReadExt, WriteExt},
    net::{TcpListener, TcpStream},
};

use futures::StreamExt;

#[derive(Debug)]
enum ProgramMode {
    Send,
    Receive,
}

async fn get_program_mode() -> ProgramMode {
    loop {
        println!("Please enter 'Send' or 'Receive':");
        io::stdout().flush().await.unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .await
            .expect("Failed to read line");

        match input.trim() {
            "Send" => return ProgramMode::Send,
            "Receive" => return ProgramMode::Receive,
            _ => println!("Invalid input"),
        }
    }
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mode = get_program_mode().await;

    match mode {
        ProgramMode::Send => handle_send().await?,
        ProgramMode::Receive => handle_receive().await?,
    };

    Ok(())
}

async fn handle_send() -> std::io::Result<()> {
    println!("Please enter the port to send data:");

    let mut port_input = String::new();
    io::stdin()
        .read_line(&mut port_input)
        .await
        .expect("Failed to read line");

    let port: u16 = port_input.trim().parse().expect("Invalid port number");

    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;

    stream.write_all(b"CONNECTED").await?;
    stream.shutdown(Shutdown::Write)?;

    loop {
        let mut buffer = String::new();
        let _ = stream.read_to_string(&mut buffer).await;
        if !buffer.is_empty() {
            println!("Received data: {}", buffer);
            break;
        }
    }

    Ok(())
}

async fn handle_receive() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    listener
        .incoming()
        .for_each_concurrent(None, |stream| async {
            if let Ok(stream) = stream {
                handle_incoming_stream(stream).await;
            }
        })
        .await;

    Ok(())
}

async fn handle_incoming_stream(stream: TcpStream) {
    let (reader, writer) = &mut (&stream, &stream);
    let mut reader = async_std::io::BufReader::new(reader);

    let mut buffer = String::new();
    let _ = reader.read_to_string(&mut buffer).await;
    if buffer == "CONNECTED" {
        println!("Connected to server");
        let address = stream.peer_addr().unwrap();
        writer
            .write_all(address.to_string().as_bytes())
            .await
            .unwrap();
    }
}
