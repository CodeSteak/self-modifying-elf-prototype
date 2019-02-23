use crate::prelude::*;

use std::fs::*;
use std::io::*;

use std::sync::*;

lazy_static! {
    pub static ref DATA: Arc<Mutex<OwnExecutable>> = {
        Arc::new(Mutex::new(
            OwnExecutable::choose().expect("Unable to open file."),
        ))
    };
}

pub struct OwnExecutable {
    offset: u64,
    file: File,
    lock: bool,
    unread: bool,
}

#[service(("core","file","write"))]
pub fn write((op,): (WriteOperation,)) -> Option<bool> {
    Some(DATA.lock().unwrap().write(&op).is_ok())
}

impl OwnExecutable {
    pub fn new() -> Result<Self> {
        let offset: u64 = std::env::var("SELF_OFFSET")
            .unwrap()
            .parse()
            .expect("SELF_OFFSET Envvar not found!");

        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(std::env::args().nth(0).unwrap())?;

        let own = OwnExecutable {
            file,
            offset,
            lock: true,
            unread: true,
        };

        Ok(own)
    }

    pub fn choose() -> Result<Self> {
        if std::env::var("SELF_OFFSET").is_ok() {
            Self::new()
        } else {
            Self::fake()
        }
    }

    pub fn fake() -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open("data_file")?;

        let own = OwnExecutable {
            file,
            offset: 0,
            lock: true,
            unread: true,
        };

        Ok(own)
    }
}

impl OwnExecutable {
    pub fn read(&mut self) -> Result<State> {
        self.lock = false;
        self.unread = false;

        let mut state: State = Default::default();

        let mut file = BufReader::new(&mut self.file);

        let end = file.seek(SeekFrom::End(0))?;
        let mut cur = file.seek(SeekFrom::Start(self.offset))?;

        loop {
            if cur == end {
                break;
            };

            match cbor::from_stream_reader::<WriteOperation, _>(&mut file) {
                Ok(d) => {
                    d.apply(&mut state);
                    cur = file.seek(SeekFrom::Current(0))?;
                }
                Err(e) => {
                    eprintln!("File is broken. Opening readonly. [{}]", e);
                    self.lock = true;
                    break;
                }
            }
        }

        Ok(state)
    }

    pub fn write(&mut self, op: &WriteOperation) -> Result<()> {
        if self.unread {
            let _ = self.read();
        }
        let mut file = BufWriter::new(&mut self.file);

        if self.lock {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "File is broken. Read only!",
            ));
        }

        cbor::to_writer(&mut file, op).expect("TODO");

        Ok(())
    }
}
