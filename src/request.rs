//request.rs

use std::collections::HashMap;
use std::net;

use uuid::{self, Uuid};
use rustc_serialize::json;

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum RequestType {
    Knock,
    Connect,
    Disconnect,
    Heartbeat,
}

//update this when adding new connection types
pub const REQUEST_TYPE_VERB_MAP: [(&'static str, RequestType); 4] = [
    ("", RequestType::Knock),
    ("CONNECT", RequestType::Connect),
    ("DISCONNECT", RequestType::Disconnect),
    ("HEARTBEAT", RequestType::Heartbeat)
];

#[derive(Clone)]
pub struct Request {
    pub id: Uuid,
    pub client_id: Uuid,
    pub req_type: RequestType,
    pub contents: Vec<u8>,
    pub addr: net::SocketAddr,
}

impl Request {
    pub fn new(serialized: String, type_hashes: HashMap<Uuid, RequestType>, stream: &net::TcpStream) -> Option<Request> {
        let req: SerializableRequest = match json::decode(serialized.as_str()) {
            Ok(r) => r,
            Err(_) => panic!("Encountered malformed request JSON: {}", serialized),
        };
        let req_type_hash: Uuid = req.req_type;
        if type_hashes.contains_key(&req_type_hash) {
            Some(Request {
                id: Uuid::new_v4(),
                client_id: req.client_id,
                req_type: type_hashes[&req_type_hash].clone(),
                contents: req.contents,
                addr: stream.peer_addr().unwrap(), //store the socket address from the tcpstream
            })
        } else {
            None
        }
    }

    //hash-map (Uuid -> RequestType) for use by server to find a RequestType given a Uuid
    pub fn create_request_type_hashes(validation_token: &str) -> HashMap<Uuid, RequestType> {
        let mut hashes: HashMap<Uuid, RequestType> = HashMap::new();
        //TODO: figure out why the linter says I need to write "ref t" here
        for &(s, ref t) in &REQUEST_TYPE_VERB_MAP {
            if t == &RequestType::Knock {
                //if a request contains a null-value uuid in its request field,
                //  equate it with the "Knock" request type in order to
                //  provide the opportunity to respond with the server's
                //  validation token.
                //the client is asking "How do I talk to you?"
                hashes.insert(Uuid::nil(), t.clone());
            } else {
                let verb_token = format!("{}_{}", s, validation_token);
                let hash = Uuid::new_v5(&uuid::NAMESPACE_OID, verb_token.as_str());
                hashes.insert(hash, t.clone());
            }
        }
        hashes
    }

    //hash-map (RequestType -> Uuid) for use by client to find a Uuid for a given RequestType
    pub fn create_request_type_reverse_hashes(validation_token: &str) -> HashMap<RequestType, Uuid> {
        let mut hashes: HashMap<RequestType, Uuid> = HashMap::new();
        for &(s, ref t) in &REQUEST_TYPE_VERB_MAP {
            if t == &RequestType::Knock {
                hashes.insert(t.clone(), Uuid::nil());
            } else {
                let verb_token = format!("{}_{}", s, validation_token);
                let hash = Uuid::new_v5(&uuid::NAMESPACE_OID, verb_token.as_str());
                hashes.insert(t.clone(), hash);
            }
        }
        hashes
    }
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct SerializableRequest {
    pub client_id: Uuid,
    pub req_type: Uuid,
    pub contents: Vec<u8>,
}
