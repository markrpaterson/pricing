mod market;
pub mod market_data;

use std::marker::PhantomData;

pub use crate::market_data::l1::L1MarketData;

use market::Market;

type PricingSourceId = u64;

pub trait PricingSource<I, A, P> {
    fn get_id(&self) -> PricingSourceId;
    fn get_instrument(&self) -> I;

    fn get_market(&self) -> &dyn Market<A, P>;
}

pub struct VenuePricingSource<I, M, A, P>
where
    M: Market<A, P>,
{
    id: PricingSourceId,
    instrument: I,
    market: M,
    phantom_amount: PhantomData<A>,
    phantom_price: PhantomData<P>,
}

impl<I, M, A, P> VenuePricingSource<I, M, A, P> where M: Market<A, P> {}

impl<I, M, A, P> PricingSource<I, A, P> for VenuePricingSource<I, M, A, P>
where
    I: Copy,
    A: PartialOrd,
    P: Copy,
    M: Market<A, P>,
{
    fn get_id(&self) -> PricingSourceId {
        self.id
    }

    fn get_instrument(&self) -> I {
        self.instrument
    }

    fn get_market(&self) -> &dyn Market<A, P> {
        &self.market
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
