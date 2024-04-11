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

pub struct Market<A, P> {
    pub bids: Vec<Level<A, P>>,
    pub offers: Vec<Level<A, P>>,
}

type PricingSourceId = u64;
type SubscriptionId = u64;

type SizeSubscriptionCallback<P> = fn(price: BidOffer<P>);
type MarketSubscriptionCallback<A, P> = fn(market: Market<A, P>);

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

    fn get_price(&self, size: A) -> BidOffer<P>;
    fn get_market(&self) -> &Market<A, P>;
}

pub struct MarketSubscription<A, P> {
    id: SubscriptionId,
    callback: MarketSubscriptionCallback<A, P>,
}

pub struct TieredMarket<I, A, P> {
    id: PricingSourceId,
    instrument: I,
    market: Market<A, P>,
    market_subscriptions: RefCell<Vec<MarketSubscription<A, P>>>,
}

impl<I, A, P> TieredMarket<I, A, P> {}

impl<I, A, P> PricingSource<I, A, P> for TieredMarket<I, A, P>
where
    I: Copy,
    A: PartialOrd,
    P: Copy,
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

    fn subscribe_to_size(&self, size: A, callback: SizeSubscriptionCallback<P>) -> Subscription {
        todo!()
    }

    fn unsubscribe(&self, handle: Subscription) {
        if handle.pricing_source == self.id {
            let mut market_subscriptions = self.market_subscriptions.borrow_mut();

            market_subscriptions.retain(|x| x.id != handle.subscription);
        }
    }

    fn get_price(&self, size: A) -> BidOffer<P> {
        let bid = self
            .market
            .bids
            .iter()
            .find(|&a| a.size > size)
            .map(|x| x.price);
        let offer = self
            .market
            .offers
            .iter()
            .find(|&a| a.size > size)
            .map(|x| x.price);

        BidOffer::new(bid, offer)
    }

    fn get_market(&self) -> &Market<A, P> {
        &self.market
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
