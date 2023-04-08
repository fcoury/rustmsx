use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::time::Instant;
use std::{env, fs};

use anyhow::{anyhow, bail, Error, Result};
use handlebars::Handlebars;
use msx::slot::Slot;
use path_absolutize::*;
use serde_json::json;
use sha1::{Digest, Sha1};
use tracing::{event, span, Level};
use walkdir::WalkDir;
use xml::reader::{EventReader, XmlEvent};

use crate::internal_state::{InternalState, ReportState};

pub enum Response {
    Ok(String),
    Nok(String),
}

pub struct Client {
    pub socket: UnixStream,
    pub reader: EventReader<UnixStream>,
    pub writer: BufWriter<UnixStream>,
    pub machine_xml: PathBuf,
}

impl ReportState for Client {
    fn report_state(&mut self) -> anyhow::Result<InternalState> {
        let pc = self.send("reg pc")?.parse()?;
        let sp = self.send("reg sp")?.parse()?;
        let a = self.send("reg a")?.parse()?;
        let f = self.send("reg f")?.parse()?;
        let b = self.send("reg b")?.parse()?;
        let c = self.send("reg c")?.parse()?;
        let d = self.send("reg d")?.parse()?;
        let e = self.send("reg e")?.parse()?;
        let h = self.send("reg h")?.parse()?;
        let l = self.send("reg l")?.parse()?;
        let hl = self.send("reg hl")?.parse()?;
        let hl_contents = self
            .send(&format!("debug read memory 0x{:04X}", hl))?
            .parse()?;
        let opcode = self
            .send(&format!("debug read memory 0x{:04X}", pc))?
            .parse()?;

        Ok(InternalState {
            pc,
            sp,
            a,
            f,
            b,
            c,
            d,
            e,
            h,
            l,
            hl,
            hl_contents,
            opcode,
        })
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.send("exit");
        let _ = fs::remove_file(&self.machine_xml);
    }
}

impl Client {
    pub fn start() -> Result<bool> {
        let result = Command::new("/usr/bin/open")
            .arg("-a")
            .arg("openMSX.app")
            .output();

        if result.is_err() {
            return Ok(false);
        }

        let start_time = Instant::now();
        loop {
            if start_time.elapsed().as_secs() > 5 {
                bail!("openMSX took too long to start.");
            }

            if find_socket().is_ok() {
                return Ok(true);
            }
        }
    }

    pub fn new(slots: &Vec<Rc<dyn Slot>>) -> Result<Client, Error> {
        let machine_xml = PathBuf::new()
            .join(dirs::home_dir().unwrap())
            .join(".openMsx")
            .join("share")
            .join("machines")
            .join("RUNNER.xml");

        let template = std::fs::read_to_string("src/template.xml.handlebars")?;

        let json = serde_json::to_value(slots)?;
        let mut slots = Vec::new();
        for (n, slot) in json.as_array().unwrap().iter().enumerate() {
            let slot = slot.as_object().unwrap();
            let typ = slot.get("type").unwrap().as_str().unwrap();
            match typ {
                "RomSlot" => {
                    let rom_path = slot.get("rom_path").unwrap().as_str().unwrap();
                    let rom_path = PathBuf::from(rom_path);
                    let file = File::open(&rom_path)?;
                    let mut reader = BufReader::new(file);
                    let mut buffer = Vec::new();
                    reader.read_to_end(&mut buffer)?;

                    let mut hasher = Sha1::new();
                    hasher.update(&buffer);

                    let base = slot.get("base").unwrap().as_i64().unwrap() as u16;
                    let size = slot.get("size").unwrap().as_i64().unwrap() as u16;

                    slots.push(json!({
                        "n": n,
                        "isRom": true,
                        "base": format!("0x{:04X}", base),
                        "size": format!("0x{:04X}", size),
                        "rom_path": rom_path.absolutize().unwrap().to_str().unwrap().to_string(),
                        "sha1": format!("{:x}", hasher.finalize()),
                    }));
                }
                "EmptySlot" => slots.push(json!({
                    "n": n,
                    "isEmpty": true,
                })),
                "RamSlot" => {
                    let base = slot.get("base").unwrap().as_i64().unwrap() as u16;
                    let size = slot.get("size").unwrap().as_i64().unwrap() as u32;

                    slots.push(json!({
                        "n": n,
                        "isRam": true,
                        "base": format!("0x{:04X}", base),
                        "size": format!("0x{:05X}", size+1),
                    }));
                }
                _ => {
                    bail!("Unknown slot type: {}", typ);
                }
            }
        }

        let json = json!({
            "slots": slots,
        });
        let reg = Handlebars::new();
        let contents = reg.render_template(&template, &json).unwrap();

        fs::write(&machine_xml, contents)?;

        let span = span!(Level::DEBUG, "Client::new");
        let _enter = span.enter();

        let socket = find_socket()?;
        let socket = UnixStream::connect(socket)?;

        let writer = BufWriter::new(socket.try_clone()?);
        let mut reader = EventReader::new(socket.try_clone()?);

        loop {
            match reader.next() {
                Ok(XmlEvent::StartElement { name, .. }) if name.local_name == "openmsx-output" => {
                    event!(Level::DEBUG, "openMSX is ready.");
                    return Ok(Client {
                        socket,
                        reader,
                        writer,
                        machine_xml,
                    });
                }
                Ok(event) => {
                    event!(Level::TRACE, "xml event: {event:?}", event = event);
                }
                Err(err) => {
                    return Err(anyhow!(err));
                }
            };
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.send("set power off")?;
        self.send("machine RUNNER")?;
        self.send("debug set_bp 0x0001")?;
        self.send("set power on")?;
        self.send("reset")?;
        Ok(())
    }

    pub fn memory_dump(&mut self, start: u16, end: u16) -> anyhow::Result<String> {
        let res = self.send(&format!("showmem {} {}", start, end))?;
        Ok(res)
    }

    pub fn vram_dump(&mut self) -> anyhow::Result<String> {
        let res = self.send("showdebuggable VRAM 0000 1024")?;
        Ok(res)
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.send("set power off")?;
        Ok(())
    }

    pub fn step(&mut self) -> Result<()> {
        self.send("debug step")?;
        Ok(())
    }

    pub fn send(&mut self, command: &str) -> anyhow::Result<String> {
        match self.request(command) {
            Ok(Response::Ok(data)) => Ok(data),
            Ok(Response::Nok(data)) => {
                Err(anyhow!("openMSX error running '{}': {}", command, data))
            }
            Err(e) => Err(e),
        }
    }

    fn request(&mut self, command: &str) -> Result<Response, Error> {
        let span = span!(Level::DEBUG, "sending a command");
        let _enter = span.enter();

        self.writer.write_all(b"<command>")?;
        self.writer.write_all(command.as_bytes())?;
        self.writer.write_all(b"</command>\n")?;
        self.writer.flush()?;

        event!(Level::DEBUG, "sent command: {command}", command = command);

        let ok = loop {
            match self.reader.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) if name.local_name == "reply" => {
                    break attributes
                        .iter()
                        .find(|attr| attr.name.local_name == "result")
                        .map(|attr| attr.value.to_owned())
                        .ok_or_else(|| anyhow!("result attribute is undefined"))?;
                }
                Ok(event) => {
                    event!(Level::TRACE, "xml event: {event:?}", event = event);
                }
                Err(err) => {
                    return Err(anyhow!(err));
                }
            };
        };

        let mut data = String::new();

        loop {
            match self.reader.next() {
                Ok(XmlEvent::Characters(chunk)) => {
                    data.push_str(chunk.as_str());
                }
                Ok(XmlEvent::EndElement { name, .. }) if name.local_name == "reply" => {
                    event!(Level::DEBUG, "reply: {ok:?}. {data}", ok = ok, data = data);

                    return if ok == "ok" {
                        Ok(Response::Ok(data))
                    } else {
                        Ok(Response::Nok(data))
                    };
                }
                Ok(event) => {
                    event!(Level::TRACE, "xml event: {event:?}", event = event);
                }
                Err(err) => {
                    return Err(anyhow!(err));
                }
            }
        }
    }
}

pub fn find_socket() -> Result<PathBuf, Error> {
    let username = env::var("USER")?;
    let socket_folder_pattern = format!("openmsx-{}", username);

    for entry in WalkDir::new("/private/var/folders")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Some(dir_name) = entry.file_name().to_str() {
            if dir_name.starts_with(&socket_folder_pattern) && entry.file_type().is_dir() {
                for subentry in entry.path().read_dir()? {
                    let subentry = subentry?;
                    if let Some(socket_name) = subentry.file_name().to_str() {
                        if socket_name.starts_with("socket.") {
                            return Ok(subentry.path());
                        }
                    }
                }
            }
        }
    }

    Err(anyhow!("Socket file not found."))
}
