use serde::Serialize;

use super::{maker_order::*, obligation::*, trade_details::*, trade_engine_details::*};
use crate::common::*;
use crate::error::*;

pub struct MakerOrderBuilder<'a, T: TradeEngineSpecfiicsTrait + Clone + Serialize> {
    event_msg_client: &'a ArcClient,
    // DB

    // Trade Specific Parameters
    trade_uuid: Option<String>, // TODO: Change to UUID type
    maker_obligation: Option<MakerObligation>,
    taker_obligation: Option<TakerObligation>,
    trade_details: Option<TradeDetails>,
    engine_details: Option<TradeEngineDetails<T>>,
    pow_difficulty: Option<u64>,
}

impl<'a, T: TradeEngineSpecfiicsTrait + Clone + Serialize> MakerOrderBuilder<'a, T> {
    pub fn new(event_msg_client: &'a ArcClient, // DB
    ) -> Self {
        MakerOrderBuilder {
            event_msg_client,
            trade_uuid: Option::<String>::None,
            maker_obligation: Option::<MakerObligation>::None,
            taker_obligation: Option::<TakerObligation>::None,
            trade_details: Option::<TradeDetails>::None,
            engine_details: Option::<TradeEngineDetails<T>>::None,
            pow_difficulty: Option::<u64>::None,
        }
    }

    pub fn trade_uuid(&mut self, trade_uuid: impl Into<String>) -> &mut Self {
        self.trade_uuid = Some(trade_uuid.into());
        self
    }

    pub fn maker_obligation(&mut self, maker_obligation: impl Into<MakerObligation>) -> &mut Self {
        self.maker_obligation = Some(maker_obligation.into());
        self
    }

    pub fn taker_obligation(&mut self, taker_obligation: impl Into<TakerObligation>) -> &mut Self {
        self.taker_obligation = Some(taker_obligation.into());
        self
    }

    pub fn trade_details(&mut self, trade_details: impl Into<TradeDetails>) -> &mut Self {
        self.trade_details = Some(trade_details.into());
        self
    }

    pub fn engine_details(
        &mut self,
        engine_details: impl Into<TradeEngineDetails<T>>,
    ) -> &mut Self {
        self.engine_details = Some(engine_details.into());
        self
    }

    pub fn pow_difficulty(&mut self, pow_difficulty: impl Into<u64>) -> &mut Self {
        self.pow_difficulty = Some(pow_difficulty.into());
        self
    }

    pub fn build(&self) -> std::result::Result<MakerOrder<T>, N3xbError> {
        let Some(trade_uuid) = self.trade_uuid.as_ref() else {
      return Err(N3xbError::Other("No Trade UUID".to_string()));  // TODO: Error handling?
    };

        let Some(maker_obligation) = self.maker_obligation.as_ref() else {
      return Err(N3xbError::Other("No Maker Obligations defined".to_string()));  // TODO: Error handling?
    };

        let Some(taker_obligation) = self.taker_obligation.as_ref() else {
      return Err(N3xbError::Other("No Taker Obligations defined".to_string()));  // TODO: Error handling?
    };

        let Some(trade_details) = self.trade_details.as_ref() else {
      return Err(N3xbError::Other("No Trade Details defined".to_string()));  // TODO: Error handling?
    };

        let Some(engine_details) = self.engine_details.as_ref() else {
      return Err(N3xbError::Other("No Engine Details defined".to_string()));  // TODO: Error handling?
    };

        let pow_difficulty = self.pow_difficulty.unwrap_or_else(|| 0);

        Ok(MakerOrder::new(
            self.event_msg_client,
            trade_uuid.to_owned(),
            maker_obligation.to_owned(),
            taker_obligation.to_owned(),
            trade_details.to_owned(),
            engine_details.to_owned(),
            pow_difficulty,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;
    use iso_currency::Currency;
    use nostr_sdk::prelude::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn maker_order_builder_build() {
        let client = new_event_msg_client();
        let mut builder: MakerOrderBuilder<TestTradeEngineSpecifics> =
            MakerOrderBuilder::new(&client);

        let some_uuid_string = "Some-UUID-String";
        builder.trade_uuid(some_uuid_string);

        let maker_obligation_kind = ObligationKind::Fiat(
            Currency::CNY,
            HashSet::from([FiatPaymentMethod::WeChatPay, FiatPaymentMethod::AliPay]),
        );
        let maker_obligation_content = MakerObligationContent {
            amount: 1000000,
            amount_min: None,
        };
        builder.maker_obligation(MakerObligation {
            kind: maker_obligation_kind.clone(),
            content: maker_obligation_content.clone(),
        });

        let taker_obligation_kind = ObligationKind::Bitcoin(HashSet::from([
            BitcoinSettlementMethod::Onchain,
            BitcoinSettlementMethod::Lightning,
        ]));
        let taker_obligation_content = TakerObligationContent {
            limit_rate: Some(0.000001),
            market_offset_pct: None,
            market_oracles: None,
        };
        builder.taker_obligation(TakerObligation {
            kind: taker_obligation_kind.clone(),
            content: taker_obligation_content.clone(),
        });

        let trade_parameters = HashSet::from([
            TradeParameter::AcceptsPartialTake,
            TradeParameter::TrustedArbitration,
            TradeParameter::TrustedEscrow,
            TradeParameter::TradeTimesOut(TradeTimeOutLimit::NoTimeout),
        ]);
        let trade_details_content = TradeDetailsContent {
            maker_bond_pct: None,
            taker_bond_pct: None,
            trade_timeout: None,
        };
        builder.trade_details(TradeDetails {
            parameters: trade_parameters.clone(),
            content: trade_details_content.clone(),
        });

        let some_engine_name_str = "some-trade-mechanics";
        let some_engine_specific_str = "some-test-specific-info";
        builder.engine_details(TradeEngineDetails {
            trade_engine_name: some_engine_name_str.to_string(),
            trade_engine_specifics: TestTradeEngineSpecifics {
                test_specific_field: some_engine_specific_str.to_string(),
            },
        });

        let some_pow_difficulty: u64 = 8;
        builder.pow_difficulty(some_pow_difficulty);

        let result = builder.build();

        match result {
            Ok(maker_order) => {
                assert_eq!(maker_order.trade_uuid, some_uuid_string);
                assert_eq!(maker_order.maker_obligation.kind, maker_obligation_kind);
                assert_eq!(
                    maker_order.maker_obligation.content,
                    maker_obligation_content
                );
                assert_eq!(maker_order.taker_obligation.kind, taker_obligation_kind);
                assert_eq!(
                    maker_order.taker_obligation.content,
                    taker_obligation_content
                );
                assert_eq!(maker_order.trade_details.parameters, trade_parameters);
                assert_eq!(maker_order.trade_details.content, trade_details_content);
                assert_eq!(
                    maker_order.engine_details.trade_engine_name,
                    some_engine_name_str.to_string()
                );
                assert_eq!(
                    maker_order
                        .engine_details
                        .trade_engine_specifics
                        .test_specific_field,
                    some_engine_specific_str.to_string()
                );
            }
            Err(error) => {
                panic!(
                    "maker_order_builder_build failed on builder.build() - {}",
                    error.to_string()
                );
            }
        }
    }

    #[test]
    fn maker_order_builder_build_trade_uuid_missing() {}

    #[test]
    fn maker_order_builder_build_maker_obligation_missing() {}

    #[test]
    fn maker_order_builder_build_taker_obligation_missing() {}

    #[test]
    fn maker_order_builder_build_trade_details_missing() {}

    #[test]
    fn maker_order_builder_build_engine_details_missing() {}

    #[test]
    fn maker_order_builder_build_pow_difficulty_missing() {}

    // Helper Definitions

    #[derive(Clone, Debug, Deserialize, Serialize)]
    struct TestTradeEngineSpecifics {
        test_specific_field: String,
    }

    #[typetag::serde(name = "test-trade-engine")]
    impl TradeEngineSpecfiicsTrait for TestTradeEngineSpecifics {}

    // Helper Functions

    fn new_event_msg_client() -> ArcClient {
        let keys = Keys::generate();
        let opts = Options::new()
            .wait_for_connection(true)
            .wait_for_send(true)
            .difficulty(8);
        let client = Client::with_opts(&keys, opts);
        Arc::new(Mutex::new(client))
    }
}
