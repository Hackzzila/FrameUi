use std::{sync::Arc, thread};

use devtools_protocol as dt;

use dashmap::DashMap;
use futures_util::sink::SinkExt;
use indextree::{Arena, NodeId};
use log::{error, trace};
use tokio::{
  net::{TcpListener, TcpStream, ToSocketAddrs},
  runtime::Runtime,
  stream::StreamExt,
};
use tungstenite::{
  handshake::server::{Request, Response},
  protocol::Message,
};

use ::dom::{CompiledDocument, Element, ElementData};

#[derive(PartialEq, Debug)]
#[repr(u16)]
#[allow(dead_code)]
enum NodeType {
  Element = 1,
  Attribute = 2,
  Text = 3,
  CDataSection = 4,
  EntityReference = 5, // historical
  Entity = 6,          // historical
  ProcessingInstruction = 7,
  Comment = 8,
  Document = 9,
  DocumentType = 10,
  DocumentFragment = 11,
  Notation = 12, // historical
}

fn node_from_element(node_id: NodeId, parent: Option<NodeId>, elements: &Arena<Element>) -> dt::dom::Node {
  let children: Vec<dt::dom::Node> = node_id
    .children(elements)
    .map(|x| node_from_element(x, Some(node_id), elements))
    .collect();

  let node = elements.get(node_id).unwrap().get();

  let node_name = node.get_local_name().to_string();

  let node_type = match node.data {
    ElementData::Root(..) => NodeType::Document,
    _ => NodeType::Element,
  };

  let node_value = String::new();

  dt::dom::Node {
    node_id: Into::<usize>::into(node_id) as i64,
    backend_node_id: Into::<usize>::into(node_id) as i64,
    node_type: node_type as i64,
    local_name: node_name.clone(),
    node_name,
    node_value,
    children: Some(children),
    parent_id: parent.map(|x| Into::<usize>::into(x) as i64),

    attributes: None,
    base_url: None,
    child_node_count: None,
    content_document: None,
    distributed_nodes: None,
    document_url: None,
    frame_id: None,
    imported_document: None,
    internal_subset: None,
    is_svg: None,
    name: None,
    pseudo_elements: None,
    pseudo_type: None,
    public_id: None,
    shadow_root_type: None,
    shadow_roots: None,
    system_id: None,
    template_content: None,
    value: None,
    xml_version: None,
  }
}

pub struct DevTools {
  counter: usize,
  documents: Arc<DashMap<usize, Arc<CompiledDocument>>>,
}

impl DevTools {
  pub fn new<T: ToSocketAddrs + Send + 'static>(addr: T) -> DevTools {
    let documents = Arc::new(DashMap::new());

    let cloned_views = Arc::clone(&documents);
    thread::spawn(move || {
      let mut rt = Runtime::new().unwrap();

      rt.block_on(async move {
        let try_socket = TcpListener::bind(addr).await;
        let mut listener = try_socket.expect("Failed to bind");

        while let Ok((stream, ..)) = listener.accept().await {
          tokio::spawn(DevTools::handle_connection(stream, Arc::clone(&cloned_views)));
        }
      });
    });

    DevTools { counter: 0, documents }
  }

  async fn handle_connection(stream: TcpStream, views: Arc<DashMap<usize, Arc<CompiledDocument>>>) {
    let mut idx = 0;
    let callback = |req: &Request, response: Response| {
      idx = match req.uri().path()[1..].parse() {
        Ok(x) => x,
        Err(_) => {
          return Err(Response::builder().status(404).body(None).unwrap());
        }
      };

      if views.contains_key(&idx) {
        Ok(response)
      } else {
        Err(Response::builder().status(404).body(None).unwrap())
      }
    };

    match tokio_tungstenite::accept_hdr_async(stream, callback).await {
      Ok(mut ws_stream) => {
        while let Some(Ok(msg)) = ws_stream.next().await {
          if let Message::Text(text) = msg {
            let msg: Result<dt::Command, _> = serde_json::from_str(&text);
            trace!("{:#?}", msg);

            match msg {
              Ok(cmd) => {
                let id = cmd.id;
                match cmd.data {
                  dt::CommandData::DOM(cmd) => match cmd {
                    dt::dom::Command::GetDocument { .. } => {
                      let out = {
                        let view = { Arc::clone(views.get(&idx).unwrap().value()) };

                        let elements = view.elements.read().unwrap();
                        let root = node_from_element(view.root, None, &elements);

                        dt::CommandResult {
                          id,
                          result: dt::CommandResultData::DOM(dt::dom::CommandResult::GetDocument {
                            root: Box::new(root),
                          }),
                        }
                      };

                      ws_stream
                        .send(Message::Text(serde_json::to_string(&out).unwrap()))
                        .await
                        .unwrap();
                    }

                    _ => {}
                  },

                  _ => {}
                }
              }

              Err(e) => {
                println!("{} {:?}", text, e);
              }
            };
          }
        }
      }

      Err(e) => error!("websocket error: {}", e),
    }
  }

  pub fn add_view(&mut self, view: Arc<CompiledDocument>) {
    self.documents.insert(self.counter, view);
    self.counter += 1;
  }
}
