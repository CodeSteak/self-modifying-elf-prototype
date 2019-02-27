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

//#[service("core","file","write")]
pub fn write(_ctx: (), op: WriteOperation) -> bool {
    DATA.lock().unwrap().write(&op).is_ok()
}

#[service("core", "own_executable", "read")]
pub fn read_binary(_ctx: ()) -> std::result::Result<ByteBuf, String> {
    read_binary_inner()
}

fn read_binary_inner() -> std::result::Result<ByteBuf, String> {
    let mut file =
        File::open(std::env::args().nth(0).unwrap()).expect("Unable to read own Executable!");

    let offset = std::env::var("SELF_OFFSET");

    if let Ok(size) = offset {
        let size: usize = size.parse().expect("Invalid SELF_OFFSET env var.");
        let mut vec = vec![0u8; size];
        let len = file.read(&mut vec[..]).map_err(|e| format!("{:?}", e))?;

        if len != size as usize {
            return Err(format!(
                "{:?}",
                Error::new(ErrorKind::UnexpectedEof, "Could not read full binary.",)
            ));
        }

        Ok(vec.into())
    } else {
        let mut vec = vec![];
        let _len = file.read_to_end(&mut vec).map_err(|e| format!("{:?}", e))?;

        Ok(vec.into())
    }
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

        cbor::to_writer(&mut file, op).expect("Could not write to file.");

        Ok(())
    }
}
