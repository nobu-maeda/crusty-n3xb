use super::{maker_order_note::*, obligation::*, trade_details::*, trade_engine_details::*};
use crate::{common::*, nostr::*};
use serde::Serialize;

pub struct Order<T: TradeEngineSpecfiicsTrait + Clone + Serialize> {
    pub trade_uuid: String, // TODO: Change to UUID type
    pub maker_obligation: MakerObligation,
    pub taker_obligation: TakerObligation,
    pub trade_details: TradeDetails,
    pub engine_details: TradeEngineDetails<T>,
    pub pow_difficulty: u64,
}

impl<T: TradeEngineSpecfiicsTrait + Clone + Serialize> Order<T> {
    // pub async fn send_event_note(&self) {
    //     // Create Note Content
    //     let maker_order_note = MakerOrderNote {
    //         maker_obligation: self.maker_obligation.content.to_owned(),
    //         taker_obligation: self.taker_obligation.content.to_owned(),
    //         trade_details: self.trade_details.content.to_owned(),
    //         trade_engine_specifics: self.engine_details.trade_engine_specifics.to_owned(),
    //         pow_difficulty: self.pow_difficulty,
    //     };

    //     let content_string = serde_json::to_string(&maker_order_note).unwrap(); // TODO: Error Handling?

    //     // Create Note Tags
    //     let mut tag_set: Vec<OrderTag> = Vec::new();

    //     tag_set.push(OrderTag::TradeUUID(self.trade_uuid.clone()));
    //     tag_set.push(OrderTag::MakerObligations(
    //         self.maker_obligation.kind.to_tags(),
    //     ));
    //     tag_set.push(OrderTag::TakerObligations(
    //         self.taker_obligation.kind.to_tags(),
    //     ));
    //     tag_set.push(OrderTag::TradeDetailParameters(
    //         self.trade_details.parameters_to_tags(),
    //     ));
    //     tag_set.push(OrderTag::TradeEngineName(
    //         self.engine_details.trade_engine_name.clone(),
    //     ));
    //     tag_set.push(OrderTag::EventKind(EventKind::MakerOrder));
    //     tag_set.push(OrderTag::ApplicationTag(N3XB_APPLICATION_TAG.to_string()));

    //     // NIP-78 Event Kind - 30078
    //     let builder = EventBuilder::new(
    //         Kind::ParameterizedReplaceable(30078),
    //         content_string,
    //         &create_event_tags(tag_set),
    //     );

    //     let keys = self.event_msg_client.lock().unwrap().keys();
    //     self.send_event_note
    //         .lock()
    //         .unwrap()
    //         .send_event(builder.to_event(&keys).unwrap())
    //         .await
    //         .unwrap();
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::super::common::tests::*;
//     use super::*;
//     use std::sync::{Arc, Mutex};

//     fn send_event_expectation(event: Event) -> Result<EventId, Error> {
//         print!("Nostr Event: {:?}", event); // TODO: Actually validate the event, Tags and JSON content is as expected
//         Result::Ok(event.id)
//     }

//     #[tokio::test]
//     async fn order_send_maker_note() {
//         let mut client: Client = Client::new();
//         client.expect_keys().returning(|| Keys::generate());
//         client.expect_send_event().returning(send_event_expectation);

//         let arc_client = Arc::new(Mutex::new(client));

//         let maker_obligation = MakerObligation {
//             kind: SomeTestParams::maker_obligation_kind(),
//             content: SomeTestParams::maker_obligation_content(),
//         };

//         let taker_obligation = TakerObligation {
//             kind: SomeTestParams::taker_obligation_kind(),
//             content: SomeTestParams::taker_obligation_content(),
//         };

//         let trade_details = TradeDetails {
//             parameters: SomeTestParams::trade_parameters(),
//             content: SomeTestParams::trade_details_content(),
//         };

//         let engine_details = TradeEngineDetails {
//             trade_engine_name: SomeTestParams::engine_name_str(),
//             trade_engine_specifics: SomeTradeEngineSpecifics {
//                 test_specific_field: SomeTestParams::engine_specific_str(),
//             },
//         };

//         let order = Order::new(
//             &arc_client,
//             SomeTestParams::some_uuid_string(),
//             maker_obligation,
//             taker_obligation,
//             trade_details,
//             engine_details,
//             SomeTestParams::pow_difficulty(),
//         );

//         order.send_event_note().await;
//     }
// }
