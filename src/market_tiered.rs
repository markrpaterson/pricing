
pub struct L2MarketData<A, P> {
    bid_pricing: Hashmap<P, A>,
    offer_pricing: Hashmap<P, A>,
}

pub struct L3MarketData<A, P, X> {
    bid_pricing: Hashmap<X, Level<A, P>>,
    offer_pricing: Hashmap<X, Level<A, P>>,
}

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

    fn get_prices(&self, sizes: &Vec<A>) -> Vec<(A, BidOffer<P>)> {
        sizes
            .into_iter()
            .map(|size| (size, self.get_price(size)))
            .collect()
    }
}
