#![feature(proc_macro_hygiene, decl_macro)]

extern crate base64;
#[macro_use]
extern crate rocket;
extern crate serde;

use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::collections::HashMap;
use rocket::State;
use rocket_contrib::json::Json;

struct InMemoryState {
    state: RwLock<HashMap::<String, Vec<u8>>>
}

#[derive(Deserialize, Serialize)]
struct Rom {
    id: String,
    program_state: String
}

#[post("/initialise", format = "json", data = "<rom>")]
fn initialise(rom: Json<Rom>, in_memory_state: State<InMemoryState>) {
    let mut writer = in_memory_state.state.write().unwrap();
    writer.insert(rom.id.clone(), base64::decode(rom.program_state.clone()).unwrap());
}

#[post("/writeByte?<id>&<address>&<value>")]
fn write_byte(id: String, address: u16, value: u8, in_memory_state: State<InMemoryState>) -> () {
    let mut writer = in_memory_state.state.write().unwrap();
    let rom = writer.get_mut(&id).unwrap();

    rom[address as usize] = value;
}

#[get("/readByte?<id>&<address>")]
fn read_byte(id: String, address: u16, in_memory_state: State<InMemoryState>) -> String {
    let reader = in_memory_state.state.read().unwrap();
    format!("{}", reader.get(&id).unwrap()[address as usize])
}

#[get("/readRange?<id>&<address>&<length>")]
fn read_range(id: String, address: u16, length: u16, in_memory_state: State<InMemoryState>) -> String {
    let reader = in_memory_state.state.read().unwrap();
    let data = reader.get(&id).unwrap();
    let address_range = (address as usize)..(address as usize + length as usize);
    format!("{}", base64::encode(&data[address_range]))
}

#[get("/status")]
fn status() -> &'static str {
    "Healthy"
}

fn rocket() -> rocket::Rocket {
    let in_memory_state = InMemoryState {
        state: RwLock::new(HashMap::<String, Vec<u8>>::new())
    };

    rocket::ignite()
        .mount("/api/v1", routes![read_byte, read_range, write_byte, initialise])
        .mount("/", routes![status])
        .manage(in_memory_state)
}

fn main() {
    rocket().launch();
}

#[cfg(test)]
mod test {
    use super::Rom;
    use super::rocket;
    use rocket::local::Client;
    use rocket::http::ContentType;
    use rocket::http::Status;

    #[test]
    fn test_status() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let mut response = client.get("/status").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("Healthy".into()));
    }

    #[test]
    fn test_read_range() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let program_state = Box::new(Rom { id: "abcd".to_string(), program_state: base64::encode(vec![0x0; 0x10000]) });
        let rom = serde_json::to_string(program_state.as_ref()).unwrap();
        let initialise_response = client
            .post("/api/v1/initialise")
            .header(ContentType::JSON)
            .body(rom)
            .dispatch();
        assert_eq!(initialise_response.status(), Status::Ok);

        let write_response1 = client
            .post(format!("/api/v1/writeByte?id={}&address={}&value={}", "abcd", 0x10, 0xFF))
            .body("")
            .dispatch();
        assert_eq!(write_response1.status(), Status::Ok);
        let write_response2 = client
            .post(format!("/api/v1/writeByte?id={}&address={}&value={}", "abcd", 0x1A, 0xFE))
            .body("")
            .dispatch();
        assert_eq!(write_response2.status(), Status::Ok);

        let mut read_range_response = client
            .get(format!("/api/v1/readRange?id={}&address={}&length={}", "abcd", 0x10, 0x10))
            .dispatch();
        assert_eq!(read_range_response.status(), Status::Ok);
        assert_eq!(read_range_response.body_string(), Some(base64::encode(vec![0xFF, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xFE, 0x0, 0x0, 0x0, 0x0, 0x0])));
    }

    #[test]
    fn test_read_write() {
        let client = Client::new(rocket()).expect("valid rocket instance");
        let program_state = Box::new(Rom { id: "abcd".to_string(), program_state: base64::encode(vec![0x0; 0x10000]) });
        let rom = serde_json::to_string(program_state.as_ref()).unwrap();
        let initialise_response = client
            .post("/api/v1/initialise")
            .header(ContentType::JSON)
            .body(rom)
            .dispatch();
        assert_eq!(initialise_response.status(), Status::Ok);

        let write_response = client
            .post(format!("/api/v1/writeByte?id={}&address={}&value={}", "abcd", 0x10, 0x8))
            .body("")
            .dispatch();
        assert_eq!(write_response.status(), Status::Ok);

        let mut read_response = client
            .get(format!("/api/v1/readByte?id={}&address={}", "abcd", 0x10))
            .dispatch();
        assert_eq!(read_response.status(), Status::Ok);
        assert_eq!(read_response.body_string(), Some("8".into()));

        let mut read_response_2 = client
            .get(format!("/api/v1/readByte?id={}&address={}", "abcd", 0x11))
            .dispatch();
        assert_eq!(read_response_2.status(), Status::Ok);
        assert_eq!(read_response_2.body_string(), Some("0".into()));
    }
}