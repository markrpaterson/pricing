use std::cell::RefCell;

pub struct Level<A, P> {
    size: A,
    price: P,
}

impl<A, P> Level<A, P> {
    fn new(size: A, price: P) -> Self {
        Self { size, price }
    }

    pub fn get_size(&self) -> &A {
        &self.size
    }

    pub fn get_price(&self) -> &P {
        &self.price
    }
}

pub struct BidOffer<P> {
    bid: Option<P>,
    offer: Option<P>,
}

impl<P> BidOffer<P> {
    fn new(bid: Option<P>, offer: Option<P>) -> Self {
        Self { bid, offer }
    }

    pub fn get_bid(&self) -> &Option<P> {
        &self.bid
    }

    pub fn get_offer(&self) -> &Option<P> {
        &self.offer
    }
}

pub trait Market<A, P> {
    fn get_price(&self, size: A) -> BidOffer<P>;
}

pub struct TieredMarket<A, P> {
    pub bids: Vec<Level<A, P>>,
    pub offers: Vec<Level<A, P>>,
}

impl<A, P> Market<A, P> for TieredMarket<A, P>
where
    A: PartialOrd,
    P: Copy,
{
    fn get_price(&self, size: A) -> BidOffer<P> {
        let bid = self
            .bids
            .iter()
            .find(|&level| level.size > size)
            .map(|level| level.price);
        let offer = self
            .offers
            .iter()
            .find(|&level| level.size > size)
            .map(|level| level.price);

        BidOffer::new(bid, offer)
    }
}

type PricingSourceId = u64;
type SubscriptionId = u64;

type SizeSubscriptionCallback<P> = fn(price: BidOffer<P>);
type MarketSubscriptionCallback<A, P> = fn(market: &dyn Market<A, P>);

pub struct Subscription {
    pricing_source: PricingSourceId,
    subscription: SubscriptionId,
}

impl Subscription {
    fn new(pricing_source: PricingSourceId, subscription: SubscriptionId) -> Self {
        Self {
            pricing_source,
            subscription,
        }
    }
}

pub trait PricingSource<I, A, P> {
    fn get_id(&self) -> PricingSourceId;
    fn get_instrument(&self) -> I;

    fn subscribe(&self, callback: MarketSubscriptionCallback<A, P>) -> Subscription;
    fn subscribe_to_size(&self, size: A, callback: SizeSubscriptionCallback<P>) -> Subscription;
    fn unsubscribe(&self, handle: Subscription);

    fn get_market(&self) -> &dyn Market<A, P>;
}

pub struct MarketSubscription<A, P> {
    id: SubscriptionId,
    callback: MarketSubscriptionCallback<A, P>,
}

pub struct VenuePricingSource<I, M, A, P>
where
    M: Market<A, P>,
{
    id: PricingSourceId,
    instrument: I,
    market: M,
    market_subscriptions: RefCell<Vec<MarketSubscription<A, P>>>,
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

    fn subscribe(&self, callback: MarketSubscriptionCallback<A, P>) -> Subscription {
        let mut market_subscriptions = self.market_subscriptions.borrow_mut();

        let subscription_id = match market_subscriptions.last() {
            Some(x) => x.id + 1,
            _ => 1,
        };

        market_subscriptions.push(MarketSubscription {
            id: subscription_id,
            callback,
        });

        Subscription::new(self.id, subscription_id)
    }

    fn subscribe_to_size(&self, _size: A, _callback: SizeSubscriptionCallback<P>) -> Subscription {
        todo!()
    }

    fn unsubscribe(&self, handle: Subscription) {
        if handle.pricing_source == self.id {
            let mut market_subscriptions = self.market_subscriptions.borrow_mut();

            market_subscriptions.retain(|x| x.id != handle.subscription);
        }
    }

    fn get_market(&self) -> &dyn Market<A, P> {
        &self.market
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
