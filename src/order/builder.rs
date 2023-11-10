use secp256k1::XOnlyPublicKey;
use uuid::Uuid;

use super::{obligation::*, order::*, trade_details::*};

use crate::common::error::*;
use crate::common::types::SerdeGenericTrait;

pub struct OrderBuilder {
    pubkey: Option<XOnlyPublicKey>,
    trade_uuid: Option<Uuid>,
    maker_obligation: Option<MakerObligation>,
    taker_obligation: Option<TakerObligation>,
    trade_details: Option<TradeDetails>,
    trade_engine_specifics: Option<Box<dyn SerdeGenericTrait>>,
    pow_difficulty: Option<u64>,
}

impl OrderBuilder {
    pub fn new() -> Self {
        OrderBuilder {
            pubkey: Option::<XOnlyPublicKey>::None,
            trade_uuid: Option::<Uuid>::None,
            maker_obligation: Option::<MakerObligation>::None,
            taker_obligation: Option::<TakerObligation>::None,
            trade_details: Option::<TradeDetails>::None,
            trade_engine_specifics: Option::None,
            pow_difficulty: Option::<u64>::None,
        }
    }

    pub fn pubkey(&mut self, pubkey: impl Into<XOnlyPublicKey>) -> &mut Self {
        self.pubkey = Some(pubkey.into());
        self
    }

    pub fn trade_uuid(&mut self, trade_uuid: impl Into<Uuid>) -> &mut Self {
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

    pub fn trade_engine_specifics(
        &mut self,
        trade_engine_specifics: Box<dyn SerdeGenericTrait>,
    ) -> &mut Self {
        self.trade_engine_specifics = Some(trade_engine_specifics);
        self
    }

    pub fn pow_difficulty(&mut self, pow_difficulty: impl Into<u64>) -> &mut Self {
        self.pow_difficulty = Some(pow_difficulty.into());
        self
    }

    pub fn build(&mut self) -> std::result::Result<Order, N3xbError> {
        let Some(pubkey) = self.pubkey.as_ref() else {
            return Err(N3xbError::Simple("No PubKey".to_string()));
        };
        let trade_uuid = if let Some(explicit_uuid) = self.trade_uuid.as_ref() {
            explicit_uuid.to_owned()
        } else {
            // Generate a new UUID
            Uuid::new_v4()
        };

        let Some(maker_obligation) = self.maker_obligation.as_ref() else {
            return Err(N3xbError::Simple("No Maker Obligations defined".to_string()));  // TODO: Error handling?
        };

        let Some(taker_obligation) = self.taker_obligation.as_ref() else {
            return Err(N3xbError::Simple("No Taker Obligations defined".to_string()));  // TODO: Error handling?
        };

        let Some(trade_details) = self.trade_details.as_ref() else {
            return Err(N3xbError::Simple("No Trade Details defined".to_string()));  // TODO: Error handling?
        };

        let Some(trade_engine_specifics) = self.trade_engine_specifics.take() else {
            return Err(N3xbError::Simple("No Trade Engine Details defined".to_string()));  // TODO: Error handling?
        };

        let pow_difficulty = self.pow_difficulty.unwrap_or_else(|| 0);

        let order = Order {
            trade_uuid: trade_uuid,
            maker_obligation: maker_obligation.to_owned(),
            taker_obligation: taker_obligation.to_owned(),
            trade_details: trade_details.to_owned(),
            trade_engine_specifics: trade_engine_specifics,
            pow_difficulty,
            _private: (),
        };

        order.validate()?;
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use core::panic;

    #[tokio::test]
    async fn test_order_builder_build() {
        let mut builder: OrderBuilder = OrderBuilder::new();

        builder.pubkey(SomeTestOrderParams::some_x_only_public_key());

        builder.trade_uuid(SomeTestOrderParams::some_uuid());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: SomeTestOrderParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: SomeTestOrderParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: SomeTestOrderParams::trade_details_content(),
        });

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });
        builder.trade_engine_specifics(trade_engine_specifics);

        builder.pow_difficulty(SomeTestOrderParams::pow_difficulty());

        let result = builder.build();

        match result {
            Ok(order) => {
                assert_eq!(order.trade_uuid, SomeTestOrderParams::some_uuid());
                assert_eq!(
                    order.maker_obligation.kinds,
                    SomeTestOrderParams::maker_obligation_kinds()
                );
                assert_eq!(
                    order.maker_obligation.content,
                    SomeTestOrderParams::maker_obligation_content()
                );
                assert_eq!(
                    order.taker_obligation.kinds,
                    SomeTestOrderParams::taker_obligation_kinds()
                );
                assert_eq!(
                    order.taker_obligation.content,
                    SomeTestOrderParams::taker_obligation_content()
                );
                assert_eq!(
                    order.trade_details.parameters,
                    SomeTestOrderParams::trade_parameters()
                );
                assert_eq!(
                    order.trade_details.content,
                    SomeTestOrderParams::trade_details_content()
                );

                let maker_order_specifics = order
                    .trade_engine_specifics
                    .downcast_ref::<SomeTradeEngineMakerOrderSpecifics>()
                    .unwrap();

                assert_eq!(
                    maker_order_specifics.test_specific_field,
                    SomeTestParams::engine_specific_str()
                );
                assert_eq!(order.pow_difficulty, SomeTestOrderParams::pow_difficulty());
            }
            Err(error) => {
                panic!(
                    "order_builder_build failed on builder.build() - {}",
                    error.to_string()
                );
            }
        }
    }

    #[tokio::test]
    async fn test_order_builder_build_trade_uuid_missing() {
        let mut builder: OrderBuilder = OrderBuilder::new();

        builder.pubkey(SomeTestOrderParams::some_x_only_public_key());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: SomeTestOrderParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: SomeTestOrderParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: SomeTestOrderParams::trade_details_content(),
        });

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });

        builder.trade_engine_specifics(trade_engine_specifics);

        builder.pow_difficulty(SomeTestOrderParams::pow_difficulty());

        let _ = builder.build().unwrap();
    }

    #[tokio::test]
    async fn test_order_builder_build_maker_obligation_missing() {
        let mut builder: OrderBuilder = OrderBuilder::new();

        builder.pubkey(SomeTestOrderParams::some_x_only_public_key());

        builder.trade_uuid(SomeTestOrderParams::some_uuid());

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: SomeTestOrderParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: SomeTestOrderParams::trade_details_content(),
        });

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });

        builder.trade_engine_specifics(trade_engine_specifics);

        builder.pow_difficulty(SomeTestOrderParams::pow_difficulty());

        let result = builder.build();

        match result {
            Ok(_) => {
                panic!("order_builder_build should not contain maker_obligation and should not result in Ok");
            }
            Err(_) => {} // TODO: Some way to check on Error returned, without hard coupling to Error handling methodology
        }
    }

    #[tokio::test]
    async fn test_order_builder_build_taker_obligation_missing() {
        let mut builder: OrderBuilder = OrderBuilder::new();

        builder.pubkey(SomeTestOrderParams::some_x_only_public_key());

        builder.trade_uuid(SomeTestOrderParams::some_uuid());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: SomeTestOrderParams::maker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: SomeTestOrderParams::trade_details_content(),
        });

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });

        builder.trade_engine_specifics(trade_engine_specifics);

        builder.pow_difficulty(SomeTestOrderParams::pow_difficulty());

        let result = builder.build();

        match result {
            Ok(_) => {
                panic!("order_builder_build should not contain taker_obligation and should not result in Ok");
            }
            Err(_) => {} // TODO: Some way to check on Error returned, without hard coupling to Error handling methodology
        }
    }

    #[tokio::test]
    async fn test_order_builder_build_trade_details_missing() {
        let mut builder: OrderBuilder = OrderBuilder::new();

        builder.pubkey(SomeTestOrderParams::some_x_only_public_key());

        builder.trade_uuid(SomeTestOrderParams::some_uuid());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: SomeTestOrderParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: SomeTestOrderParams::taker_obligation_content(),
        });

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });

        builder.trade_engine_specifics(trade_engine_specifics);

        builder.pow_difficulty(SomeTestOrderParams::pow_difficulty());

        let result = builder.build();

        match result {
            Ok(_) => {
                panic!("order_builder_build should not contain trade_details and should not result in Ok");
            }
            Err(_) => {} // TODO: Some way to check on Error returned, without hard coupling to Error handling methodology
        }
    }

    #[tokio::test]
    async fn test_order_builder_build_engine_details_missing() {
        let mut builder: OrderBuilder = OrderBuilder::new();

        builder.pubkey(SomeTestOrderParams::some_x_only_public_key());

        builder.trade_uuid(SomeTestOrderParams::some_uuid());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: SomeTestOrderParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: SomeTestOrderParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: SomeTestOrderParams::trade_details_content(),
        });

        builder.pow_difficulty(SomeTestOrderParams::pow_difficulty());

        let result = builder.build();

        match result {
            Ok(_) => {
                panic!("order_builder_build should not contain engine_details and should not result in Ok");
            }
            Err(_) => {} // TODO: Some way to check on Error returned, without hard coupling to Error handling methodology
        }
    }

    #[tokio::test]
    async fn test_order_builder_build_pow_difficulty_default() {
        let mut builder: OrderBuilder = OrderBuilder::new();

        builder.pubkey(SomeTestOrderParams::some_x_only_public_key());

        builder.trade_uuid(SomeTestOrderParams::some_uuid());

        builder.maker_obligation(MakerObligation {
            kinds: SomeTestOrderParams::maker_obligation_kinds(),
            content: SomeTestOrderParams::maker_obligation_content(),
        });

        builder.taker_obligation(TakerObligation {
            kinds: SomeTestOrderParams::taker_obligation_kinds(),
            content: SomeTestOrderParams::taker_obligation_content(),
        });

        builder.trade_details(TradeDetails {
            parameters: SomeTestOrderParams::trade_parameters(),
            content: SomeTestOrderParams::trade_details_content(),
        });

        let trade_engine_specifics = Box::new(SomeTradeEngineMakerOrderSpecifics {
            test_specific_field: SomeTestParams::engine_specific_str(),
        });

        builder.trade_engine_specifics(trade_engine_specifics);

        let result = builder.build();

        match result {
            Ok(order) => {
                assert_eq!(order.trade_uuid, SomeTestOrderParams::some_uuid());
                assert_eq!(
                    order.maker_obligation.kinds,
                    SomeTestOrderParams::maker_obligation_kinds()
                );
                assert_eq!(
                    order.maker_obligation.content,
                    SomeTestOrderParams::maker_obligation_content()
                );
                assert_eq!(
                    order.taker_obligation.kinds,
                    SomeTestOrderParams::taker_obligation_kinds()
                );
                assert_eq!(
                    order.taker_obligation.content,
                    SomeTestOrderParams::taker_obligation_content()
                );
                assert_eq!(
                    order.trade_details.parameters,
                    SomeTestOrderParams::trade_parameters()
                );
                assert_eq!(
                    order.trade_details.content,
                    SomeTestOrderParams::trade_details_content()
                );

                let maker_order_specifics = order
                    .trade_engine_specifics
                    .downcast_ref::<SomeTradeEngineMakerOrderSpecifics>()
                    .unwrap();

                assert_eq!(
                    maker_order_specifics.test_specific_field,
                    SomeTestParams::engine_specific_str()
                );
                assert_eq!(order.pow_difficulty, 0);
            }
            Err(error) => {
                panic!(
                    "order_builder_build failed on builder.build() - {}",
                    error.to_string()
                );
            }
        }
    }
}
