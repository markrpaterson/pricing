pub struct BidOffer<P> {
    bid: Option<P>,
    offer: Option<P>,
}

impl<P> BidOffer<P> {
    pub fn new(bid: Option<P>, offer: Option<P>) -> Self {
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
    fn get_prices(&self, sizes: &[A]) -> Vec<(A, BidOffer<P>)>;
}
