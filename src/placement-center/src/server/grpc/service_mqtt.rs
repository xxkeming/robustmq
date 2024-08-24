// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    cache::placement::PlacementCacheManager,
    core::share_sub::ShareSubLeader,
    raft::apply::{RaftMachineApply, StorageData, StorageDataType},
    storage::{
        mqtt::{
            acl::AclStorage, session::MQTTSessionStorage, topic::MQTTTopicStorage,
            user::MQTTUserStorage,
        },
        rocksdb::RocksDBEngine,
    },
};
use prost::Message;
use protocol::placement_center::generate::{
    common::CommonReply,
    mqtt::{
        mqtt_service_server::MqttService, CreateAclRequest, CreateSessionRequest,
        CreateTopicRequest, CreateUserRequest, DeleteAclRequest, DeleteSessionRequest,
        DeleteTopicRequest, DeleteUserRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest,
        ListAclReply, ListAclRequest, ListSessionReply, ListSessionRequest, ListTopicReply,
        ListTopicRequest, ListUserReply, ListUserRequest, SaveLastWillMessageRequest,
        SetTopicRetainMessageRequest, UpdateSessionRequest,
    },
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcMqttService {
    cluster_cache: Arc<PlacementCacheManager>,
    placement_center_storage: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcMqttService {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
        placement_center_storage: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcMqttService {
            cluster_cache,
            placement_center_storage,
            rocksdb_engine_handler,
        }
    }
}

impl GrpcMqttService {}

#[tonic::async_trait]
impl MqttService for GrpcMqttService {
    async fn get_share_sub_leader(
        &self,
        request: Request<GetShareSubLeaderRequest>,
    ) -> Result<Response<GetShareSubLeaderReply>, Status> {
        let req = request.into_inner();
        let cluster_name = req.cluster_name;
        let group_name = req.group_name;
        let mut reply = GetShareSubLeaderReply::default();
        let share_sub = ShareSubLeader::new(
            self.cluster_cache.clone(),
            self.rocksdb_engine_handler.clone(),
        );

        let leader_broker = match share_sub.get_leader_node(&cluster_name, &group_name) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };

        if let Some(node) = self
            .cluster_cache
            .get_node_addr(&cluster_name, leader_broker)
        {
            reply.broker_id = leader_broker;
            reply.broker_addr = node.node_inner_addr;
            reply.extend_info = node.extend;
        }

        return Ok(Response::new(reply));
    }

    async fn list_user(
        &self,
        request: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let req = request.into_inner();
        let storage = MQTTUserStorage::new(self.rocksdb_engine_handler.clone());

        if !req.username.is_empty() {
            match storage.get(&req.cluster_name, &req.username) {
                Ok(Some(data)) => {
                    return Ok(Response::new(ListUserReply {
                        users: vec![data.encode()],
                    }));
                }
                Ok(None) => {
                    return Ok(Response::new(ListUserReply::default()));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else {
            match storage.list(&req.cluster_name) {
                Ok(data) => {
                    let mut result = Vec::new();
                    for raw in data {
                        result.push(raw.encode());
                    }
                    return Ok(Response::new(ListUserReply { users: result }));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        let data = StorageData::new(
            StorageDataType::MQTTCreateUser,
            CreateUserRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "create_user".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        let data = StorageData::new(
            StorageDataType::MQTTDeleteUser,
            DeleteUserRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "delete_user".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn create_topic(
        &self,
        request: Request<CreateTopicRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTCreateTopic,
            CreateTopicRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "create_topic".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_topic(
        &self,
        request: Request<DeleteTopicRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTDeleteTopic,
            DeleteTopicRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "delete_topic".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<ListTopicReply>, Status> {
        let req = request.into_inner();
        let storage = MQTTTopicStorage::new(self.rocksdb_engine_handler.clone());
        if !req.topic_name.is_empty() {
            match storage.get(&req.cluster_name, &req.topic_name) {
                Ok(Some(data)) => {
                    return Ok(Response::new(ListTopicReply {
                        topics: vec![data.encode()],
                    }));
                }
                Ok(None) => {
                    return Ok(Response::new(ListTopicReply::default()));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else {
            match storage.list(&req.cluster_name) {
                Ok(data) => {
                    let mut result = Vec::new();
                    for raw in data {
                        result.push(raw.encode());
                    }
                    return Ok(Response::new(ListTopicReply { topics: result }));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
    }

    async fn list_session(
        &self,
        request: Request<ListSessionRequest>,
    ) -> Result<Response<ListSessionReply>, Status> {
        let req = request.into_inner();
        let storage = MQTTSessionStorage::new(self.rocksdb_engine_handler.clone());

        if !req.client_id.is_empty() {
            match storage.get(&req.cluster_name, &req.client_id) {
                Ok(Some(data)) => {
                    return Ok(Response::new(ListSessionReply {
                        sessions: vec![data.encode()],
                    }));
                }
                Ok(None) => {
                    return Ok(Response::new(ListSessionReply::default()));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else {
            match storage.list(&req.cluster_name) {
                Ok(data) => {
                    let mut result = Vec::new();
                    for raw in data {
                        result.push(raw.data);
                    }
                    let reply = ListSessionReply { sessions: result };
                    return Ok(Response::new(reply));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTCreateSession,
            CreateSessionRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "create_session".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTDeleteSession,
            DeleteSessionRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "delete_session".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn set_topic_retain_message(
        &self,
        request: Request<SetTopicRetainMessageRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTSetTopicRetainMessage,
            SetTopicRetainMessageRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "set_topic_retain_message".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn update_session(
        &self,
        request: Request<UpdateSessionRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTUpdateSession,
            UpdateSessionRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "update_session".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn save_last_will_message(
        &self,
        request: Request<SaveLastWillMessageRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTSaveLastWillMessage,
            SaveLastWillMessageRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "save_last_will_message".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
    async fn list_acl(
        &self,
        request: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        let req = request.into_inner();
        let acl_storage = AclStorage::new(self.rocksdb_engine_handler.clone());
        match acl_storage.list(&req.cluster_name) {
            Ok(list) => {
                let mut acls = Vec::new();
                for acl in list {
                    match acl.encode() {
                        Ok(data) => {
                            acls.push(data);
                        }
                        Err(e) => {
                            return Err(Status::cancelled(e.to_string()));
                        }
                    }
                }

                return Ok(Response::new(ListAclReply { acls }));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTCreateAcl,
            CreateAclRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "mqtt_create_acl".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTDeleteAcl,
            DeleteAclRequest::encode_to_vec(&req),
        );

        match self
            .placement_center_storage
            .apply_propose_message(data, "mqtt_delete_acl".to_string())
            .await
        {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
