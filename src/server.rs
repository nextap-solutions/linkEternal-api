use std::collections::HashMap;

use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

use link_eternal::{
    link_service_server::{LinkService, LinkServiceServer},
    AddLinkRequest, ListLinksRequest, ListLinksResponse
};

pub mod link_eternal {
    tonic::include_proto!("api");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("api_descriptor");
}

#[derive(Debug, Default)]
pub struct EternalLinkService {
    pub links: RwLock<HashMap<String, link_eternal::Link>>,
}

#[tonic::async_trait]
impl LinkService for EternalLinkService {
    async fn list_links(
        &self,
        request: Request<ListLinksRequest>,
    ) -> Result<Response<ListLinksResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = link_eternal::ListLinksResponse {
            data: self.links.read().await.clone().values().cloned().collect(),
        };

        Ok(Response::new(reply))
    }

    async fn add_link(
        &self,
        request: Request<AddLinkRequest>,
    ) -> Result<Response<link_eternal::Link>, Status> {
        println!("Got a request: {:?}", request);

        let mut self_links = self.links.write().await;

        let new_id = Uuid::new_v4(); 

        let add_link_request = request.into_inner();
        let new_link = link_eternal::Link {
            id: new_id.to_string(),
            url: add_link_request.url,
            tags: add_link_request.tags,
            description: add_link_request.description,
        };
        self_links.insert(new_id.to_string(), new_link.clone());

        let reply = new_link.clone();

        Ok(Response::new(reply))
    }

    async fn delete_link(
        &self,
        request: Request<link_eternal::DeleteLinkRequest>,
    ) -> Result<Response<link_eternal::DeleteLinkResponse>, Status> {
        println!("Got a request: {:?}", request);

        let mut self_links = self.links.write().await;
        self_links.remove(&request.into_inner().id);

        Ok(Response::new(link_eternal::DeleteLinkResponse {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(link_eternal::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let addr = "[::1]:50051".parse()?;
    let greeter = EternalLinkService::default();

    Server::builder()
        .add_service(service)
        .add_service(LinkServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
