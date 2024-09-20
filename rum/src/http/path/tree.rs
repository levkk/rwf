use super::super::Handler;
use super::{Part, Path};

pub struct Node<'a> {
    children: Vec<Node<'a>>,
    part: Part<'a>,
    handler: Option<Handler>,
}

impl<'a> Node<'a> {
    pub fn new(part: Part<'a>, handler: Option<Handler>) -> Self {
        Self {
            children: vec![],
            part,
            handler,
        }
    }

    pub fn find(&'a self, path: &Path) -> Option<&'a Handler> {
        self.traverse(&path.parts())
            .map(|n| n.handler.as_ref().unwrap())
    }

    pub fn insert(&'a mut self, path: &Path) -> Node<'a> {
        let parts = path.parts();

        todo!()
    }

    // fn insert_internal(&'a mut self, mut parts: Vec<Part<'a>>) -> Option<&mut Node<'a>> {
    //     if let Some(part) = parts.pop() {
    //         for child in self.children.iter_mut() {
    //             if child.part == part {
    //                 let node = self.insert_internal(parts);

    //                 if let Some(node) = node {
    //                     return Some(node);
    //                 } else {
    //                     return Some(self);
    //                 }
    //             }
    //         }

    //         let mut node = Node {
    //             part,
    //             children: vec![],
    //             handler: None,
    //         };

    //         let leaf = node.insert_internal(parts);
    //         self.children.push(node);

    //         leaf
    //     } else {
    //         None
    //     }
    // }

    pub fn traverse(&self, parts: &[Part<'_>]) -> Option<&'a Node> {
        if let Some(part) = parts.first() {
            // println!("parts: {:?}, part: {:?}", parts, part);
            if *part == self.part {
                let parts = &parts[1..];
                for child in &self.children {
                    // println!("travering child: {:?}", child.part);
                    match child.traverse(parts) {
                        Some(node) => match node.handler {
                            Some(_) => {
                                return Some(node);
                            }
                            None => {
                                // println!("part {:?} has no handler", node.part);
                            }
                        },
                        None => {
                            // println!("traverse {:?} returned nothing", child.part);
                        }
                    }
                }

                if self.handler.is_some() {
                    // println!("returning self: {:?}", self.part);
                    Some(self)
                } else {
                    None
                }
            } else {
                // println!("couldn't find part");
                None
            }
        } else {
            // println!("empty parts");
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::Path;
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

    #[tokio::test]
    async fn test_path_finder() {
        let root = Node {
            children: vec![Node {
                part: Part::Segment("api"),
                handler: Some(Handler::new("/api", Box::new(ControllerOne {}))),
                children: vec![Node {
                    part: Part::Slash,
                    handler: None,
                    children: vec![Node {
                        part: Part::Segment("orders"),
                        handler: Some(Handler::new("/api/orders", Box::new(ControllerTwo {}))),
                        children: vec![],
                    }],
                }],
            }],
            part: Part::Slash,
            handler: None,
        };

        let path = Path::parse("/api").unwrap();
        let handler = root.find(&path).expect("to have a handler");
        assert_eq!(
            handler
                .handle(&Request::default())
                .await
                .unwrap()
                .as_str()
                .unwrap(),
            "ControllerOne"
        );

        let path = Path::parse("/api/orders").unwrap();
        let handler = root.find(&path).expect("to have a handler");
        assert_eq!(
            handler
                .handle(&Request::default())
                .await
                .unwrap()
                .as_str()
                .unwrap(),
            "ControllerTwo"
        );
    }

    async fn get_response(node: &Node<'_>) -> String {
        let request = Request::default();
        let controller = node.handler.as_ref().expect("to have a handler");
        let response = controller.handle(&request).await.expect("response");
        let body = String::from_utf8_lossy(response.as_slice()).to_string();
        body
    }
}
