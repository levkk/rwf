use std::rc::Rc;

use super::super::Handler;
use super::Part;

pub struct Node {
    children: Vec<Node>,
    part: Part,
    handler: Option<Handler>,
}

impl Node {
    pub fn find(&self, part: &Part) -> Option<&Node> {
        let c = self
            .children
            .iter()
            .map(|c| c.part.clone())
            .collect::<Vec<_>>();
        println!("looking for part {:?} in children: {:?}", part, c);
        for node in &self.children {
            if node.part == *part {
                return Some(node);
            }
        }

        None
    }

    pub fn traverse(&self, parts: &[Part]) -> Option<&Node> {
        if let Some(part) = parts.first() {
            println!("parts: {:?}, part: {:?}", parts, part);
            if *part == self.part {
                let parts = &parts[1..];
                for child in &self.children {
                    println!("travering child: {:?}", child.part);
                    match child.traverse(parts) {
                        Some(node) => match node.handler {
                            Some(_) => {
                                return Some(node);
                            }
                            None => {
                                println!("part {:?} has no handler", node.part);
                            }
                        },
                        None => {
                            println!("traverse {:?} returned nothing", child.part);
                        }
                    }
                }

                if self.handler.is_some() {
                    println!("returning self: {:?}", self.part);
                    Some(self)
                } else {
                    None
                }
            } else {
                println!("couldn't find part");
                None
            }
        } else {
            println!("empty parts");
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::controller::{Controller, Error};
    use crate::http::{Request, Response};
    use async_trait::async_trait;

    struct ControllerOne {}
    struct ControllerTwo {}

    #[async_trait]
    impl Controller for ControllerOne {
        async fn handle(&self, _request: &Request) -> Result<Response, Error> {
            Ok(Response::text("ControllerOne"))
        }
    }

    #[async_trait]
    impl Controller for ControllerTwo {
        async fn handle(&self, _request: &Request) -> Result<Response, Error> {
            Ok(Response::text("ControllerTwo"))
        }
    }

    #[tokio::test]
    async fn test_nodes() {
        let root = Node {
            children: vec![Node {
                part: Part::Segment("orders".into()),
                handler: None,
                children: vec![Node {
                    part: Part::Segment("new".into()),
                    handler: Some(Handler::new("/api/orders/new", Box::new(ControllerTwo {}))),
                    children: vec![],
                }],
            }],
            part: Part::Segment("api".into()),
            handler: Some(Handler::new("/api", Box::new(ControllerOne {}))),
        };

        let request = Request::default();

        let path = [
            Part::Segment("api".into()),
            Part::Segment("orders".into()),
            Part::Segment("new".into()),
        ];

        let node = root.traverse(&path).expect("to have a node");
        let controller = node.handler.as_ref().expect("to have a handler");
        let response = controller.handle(&request).await.expect("response");
        let body = String::from_utf8_lossy(response.as_slice());
        assert_eq!(body, "ControllerTwo");

        let path = [Part::Segment("api".into()), Part::Segment("orders".into())];

        let node = root.traverse(&path).expect("to have a node");
        assert_eq!(get_response(node).await, "ControllerOne");
    }

    async fn get_response(node: &Node) -> String {
        let request = Request::default();
        let controller = node.handler.as_ref().expect("to have a handler");
        let response = controller.handle(&request).await.expect("response");
        let body = String::from_utf8_lossy(response.as_slice()).to_string();
        body
    }
}
