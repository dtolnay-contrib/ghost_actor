use crate::*;

ghost_actor::ghost_chan! {
    /// Incoming events from the connection.
    pub chan ConEvent<MudError> {
        fn user_command(cmd: String) -> ();
    }
}

pub type ConEventReceiver = futures::channel::mpsc::Receiver<ConEvent>;

ghost_actor::ghost_actor! {
    /// A connected mud client.
    pub actor Con<MudError> {
        fn write_raw(msg: Vec<u8>) -> ();
    }
}

pub async fn spawn_con(
    socket: tokio::net::TcpStream,
) -> (ConSender, ConEventReceiver) {
    // open a channel for the read task
    let (mut rsend, rrecv) = futures::channel::mpsc::channel(10);

    let (mut read_half, mut write_half) = socket.into_split();

    // don't echo
    write_half.write_all(&[0xff, 0xfb, 0x01]).await.unwrap();
    // don't buffer until return
    write_half.write_all(&[0xff, 0xfb, 0x03]).await.unwrap();

    // spawn the actor impl
    let (sender, driver) =
        ConSender::ghost_actor_spawn(Box::new(move |mut i_s| {
            // spawn the write task
            let write_sender = spawn_write_task(write_half);

            let mut write_sender_clone = write_sender.clone();

            // spawn the read task
            tokio::task::spawn(async move {
                let mut cmd = Vec::with_capacity(20);
                let mut wait_command = 0;
                while let Ok(c) = read_half.read_u8().await {
                    println!("char: {}", c);
                    if wait_command > 0 {
                        // mid IAC command - ignore the rest
                        wait_command -= 1;
                        continue;
                    }
                    match c {
                        255 => {
                            // IAC command - ignore the rest
                            wait_command = 2;
                        }
                        27 | 3 | 4 => {
                            i_s.ghost_actor_shutdown_immediate();
                            return;
                        }
                        8 | 127 => {
                            cmd.pop();
                            write_sender_clone
                                .set_buffer(cmd.clone())
                                .await
                                .unwrap();
                        }
                        10 | 13 => {
                            write_sender_clone
                                .set_buffer(Vec::with_capacity(0))
                                .await
                                .unwrap();
                            if let Ok(s) = std::str::from_utf8(&cmd) {
                                let s = s.trim();
                                if s.len() > 0 {
                                    rsend
                                        .user_command(s.to_string())
                                        .await
                                        .unwrap();
                                }
                            }
                            cmd.clear();
                        }
                        _ => {
                            if cmd.len() < 20 && c >= 0x20 && c <= 0x7e {
                                cmd.push(c);
                                write_sender_clone
                                    .set_buffer(cmd.clone())
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                }
            });

            async move { Ok(ConImpl { write_sender }) }.boxed().into()
        }))
        .await
        .unwrap();

    tokio::task::spawn(driver);

    (sender, rrecv)
}

ghost_actor::ghost_chan! {
    /// Incoming events from the connection.
    chan WriteControl<MudError> {
        fn set_buffer(buffer: Vec<u8>) -> ();
        fn write_line(line: Vec<u8>) -> ();
    }
}

type WriteControlSender = futures::channel::mpsc::Sender<WriteControl>;

const ERASE_LINE: &'static [u8] = &[
    13, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 13,
];

fn spawn_write_task(
    mut write_half: tokio::net::tcp::OwnedWriteHalf,
) -> WriteControlSender {
    let (wc_send, mut wc_recv) = futures::channel::mpsc::channel(10);

    tokio::task::spawn(async move {
        let mut line_buffer: Vec<u8> = Vec::new();
        while let Some(cmd) = wc_recv.next().await {
            match cmd {
                WriteControl::SetBuffer {
                    respond, buffer, ..
                } => {
                    // set our new prompt/line_buffer
                    line_buffer = buffer;
                    // move to start of line
                    write_half.write_all(ERASE_LINE).await.unwrap();
                    // write our current prompt/line_buffer
                    write_half.write_all(&line_buffer).await.unwrap();
                    // let our caller know we're done
                    respond.respond(Ok(()));
                }
                WriteControl::WriteLine { respond, line, .. } => {
                    // move to start of line
                    write_half.write_all(ERASE_LINE).await.unwrap();
                    // write the output
                    write_half.write_all(&line).await.unwrap();
                    // move to beginnig of next line
                    write_half.write_all(&[10, 13]).await.unwrap();
                    // write our current prompt/line_buffer
                    write_half.write_all(&line_buffer).await.unwrap();
                    // let our caller know we're done
                    respond.respond(Ok(()));
                }
            }
        }
    });

    wc_send
}

struct ConImpl {
    write_sender: WriteControlSender,
}

impl ConHandler<(), ()> for ConImpl {
    fn handle_write_raw(&mut self, msg: Vec<u8>) -> ConHandlerResult<()> {
        let mut write_sender = self.write_sender.clone();
        Ok(async move {
            write_sender.write_line(msg).await?;
            Ok(())
        }
        .boxed()
        .into())
    }
}
