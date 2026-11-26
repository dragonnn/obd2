use std::io;
use std::os::unix::net::UnixDatagram;

use tracing::{Level, Metadata, Subscriber};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::{MakeWriter, MakeWriterExt};

pub(super) fn subscriber() -> impl Subscriber {
    let syslog = UnixDatagram::unbound()
        .and_then(|socket| {
            socket.set_nonblocking(true)?;
            Ok(socket)
        })
        .ok();
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("obd2::rpc=debug"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_timer(tracing_subscriber::fmt::time::Uptime::default())
        .with_writer(Syslog(syslog).and(io::stderr))
        .finish()
}

struct Syslog(Option<UnixDatagram>);

impl<'a> MakeWriter<'a> for Syslog {
    type Writer = SyslogEvent<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        SyslogEvent {
            socket: self.0.as_ref(),
            severity: 6,
        }
    }

    fn make_writer_for(&'a self, metadata: &Metadata<'_>) -> Self::Writer {
        SyslogEvent {
            severity: match *metadata.level() {
                Level::ERROR => 3,
                Level::WARN => 4,
                Level::INFO => 6,
                Level::DEBUG | Level::TRACE => 7,
            },
            ..self.make_writer()
        }
    }
}

struct SyslogEvent<'a> {
    socket: Option<&'a UnixDatagram>,
    severity: u8,
}

impl io::Write for SyslogEvent<'_> {
    fn write(&mut self, line: &[u8]) -> io::Result<usize> {
        if let Some(socket) = self.socket {
            let priority = 8 + self.severity; // LOG_USER facility
            let mut message = format!("<{priority}>obd2[{}]: ", std::process::id()).into_bytes();
            message.extend_from_slice(line);
            // Logging must not stall UART replies when the syslog socket is full.
            let _ = socket.send_to(&message, "/dev/log");
        }
        Ok(line.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
