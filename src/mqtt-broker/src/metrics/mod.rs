use lazy_static::lazy_static;
use prometheus::{register_int_gauge_vec, IntGaugeVec};

const METRICS_KEY_MODULE_NAME: &str = "module";
const METRICS_KEY_PROTOCOL_NAME: &str = "protocol";
const METRICS_KEY_TYPE_NAME: &str = "type";

lazy_static! {
    static ref BROKER_PACKET_NUM: IntGaugeVec = register_int_gauge_vec!(
        "broker_packet_num",
        "broker packet num",
        &[
            METRICS_KEY_MODULE_NAME,
            METRICS_KEY_PROTOCOL_NAME,
            METRICS_KEY_TYPE_NAME,
        ]
    )
    .unwrap();
    static ref BROKER_NETWORK_QUEUE_NUM: IntGaugeVec = register_int_gauge_vec!(
        "broker_network_queue_num",
        "broker network queue num",
        &[
            METRICS_KEY_MODULE_NAME,
            METRICS_KEY_PROTOCOL_NAME,
            METRICS_KEY_TYPE_NAME,
        ]
    )
    .unwrap();
}

pub fn metrics_request_packet_incr(lable: &str) {
    BROKER_PACKET_NUM
        .with_label_values(&["broker", lable, "request"])
        .inc();
}

pub fn metrics_response_packet_incr(lable: &str) {
    BROKER_PACKET_NUM
        .with_label_values(&["broker", lable, "response"])
        .inc();
}

pub fn metrics_request_queue(lable: &str, len: i64) {
    BROKER_NETWORK_QUEUE_NUM
        .with_label_values(&["broker", lable, "request"])
        .set(len);
}

pub fn metrics_response_queue(lable: &str, len: i64) {
    BROKER_NETWORK_QUEUE_NUM
        .with_label_values(&["broker", lable, "response"])
        .set(len);
}
