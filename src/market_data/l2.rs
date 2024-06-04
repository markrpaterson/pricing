use super::{MarketSide, UpdateAction, BidOffer};
use std::{
    collections::BTreeMap,
    ops::{Add, Div, Mul, Sub},
};

/// A structure to hold L2 pricing which is Sweepable.  This means that it is that the price for any given size
/// is the VWAP of the levels required to obtain that size.
///
/// # Generic Parameters
///
/// * `A` - The amount type that should be used.
/// * `P` - The price type that should be used.
pub struct L2SweepableMarketData<P, A>
where
    P: Copy
        + PartialOrd
        + Ord
        + Add<Output = P>
        + Mul<A, Output = A>
        + Div<Output = P>
        + Default
        + From<i32>,
    A: Copy
        + PartialOrd
        + Add<Output = A>
        + Sub<Output = A>
        + Div<P, Output = A>
        + Div<A, Output = P>
        + Default
        + From<i32>,
{
    bids: BTreeMap<P, A>,
    offers: BTreeMap<P, A>,
}

impl<P, A> L2SweepableMarketData<P, A>
where
    P: Copy
        + PartialOrd
        + Ord
        + Add<Output = P>
        + Mul<A, Output = A>
        + Div<Output = P>
        + Default
        + From<i32>,
    A: Copy
        + PartialOrd
        + Add<Output = A>
        + Sub<Output = A>
        + Div<P, Output = A>
        + Div<A, Output = P>
        + Default
        + From<i32>,
{
    /// Use the new function to create a new L2SweepableMarketData with no pricing.
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L2SweepableMarketData;
    ///
    /// let market_data = L2SweepableMarketData::<i32, i32>::new();
    ///
    /// // TBC
    /// ```
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            offers: BTreeMap::new(),
        }
    }

    pub fn update(&mut self, action: UpdateAction, side: MarketSide, price: P, size: A) {
        let side_store = match side {
            MarketSide::Bid => &mut self.bids,
            MarketSide::Offer => &mut self.offers,
        };

        match action {
            UpdateAction::Add => {
                side_store.insert(price, size);
            }
            UpdateAction::Update => {
                if let Some(value) = side_store.get_mut(&price) {
                    *value = size
                };
            }
            UpdateAction::Remove => {
                side_store.remove(&price);
            }
        };
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.offers.clear();
    }

    pub fn get_price(&self, size: A) -> BidOffer<P> {
        BidOffer::new_with_price(
            self.calc_vwap(size, self.bids.iter().rev()),
            self.calc_vwap(size, self.offers.iter()),
        )
    }

    fn calc_vwap<'a, I>(&self, size: A, iter: I) -> Option<P>
    where
        I: Iterator<Item = (&'a P, &'a A)>,
        A: 'a,
        P: 'a,
    {
        let mut current_size = A::default();
        let mut current_total = A::default();

        for (&next_price, &next_size) in iter {
            let mut incremental_size = next_size;

            if next_size + current_size > size {
                incremental_size = size - current_size;
            }

            current_total = current_total + (next_price * incremental_size);
            current_size = current_size + incremental_size;

            // Should always be equal at this point but just in case of some weird rounding issues
            if current_size >= size {
                return Some(current_total / current_size);
            }
        }

        None
    }
}

impl<P, A> Default for L2SweepableMarketData<P, A>
where
    P: Copy
        + PartialOrd
        + Ord
        + Add<Output = P>
        + Mul<A, Output = A>
        + Div<Output = P>
        + Default
        + From<i32>,
    A: Copy
        + PartialOrd
        + Add<Output = A>
        + Sub<Output = A>
        + Div<P, Output = A>
        + Div<A, Output = P>
        + Default
        + From<i32>,
{
    fn default() -> Self {
        Self::new()
    }
}

/// A structure to hold L2 pricing which is Full Amount.  This means that it is that the price for any given size
/// is the price publish for the size.
///
/// # Generic Parameters
///
/// * `A` - The amount type that should be used.
/// * `P` - The price type that should be used.
pub struct L2FullAmountMarketData<P, A>
where
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
    A: Copy + Ord + PartialOrd + Add<Output = A> + Div<Output = A> + From<i32>,
{
    bids: BTreeMap<A, P>,
    offers: BTreeMap<A, P>,
}

impl<P, A> L2FullAmountMarketData<P, A>
where
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
    A: Copy + Ord + PartialOrd + Add<Output = A> + Div<Output = A> + From<i32>,
{
    /// Use the new function to create a new L2FullAmountMarketData with no pricing.
    ///
    /// # Example
    ///
    /// ```
    /// use pricing::market_data::L2FullAmountMarketData;
    ///
    /// let market_data = L2FullAmountMarketData::<i32, i32>::new();
    ///
    /// // TBC
    /// ```
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            offers: BTreeMap::new(),
        }
    }

    pub fn update(&mut self, action: UpdateAction, side: MarketSide, price: P, size: A) {
        let side_store = match side {
            MarketSide::Bid => &mut self.bids,
            MarketSide::Offer => &mut self.offers,
        };

        match action {
            UpdateAction::Add => {
                side_store.insert(size, price);
            }
            UpdateAction::Update => {
                if let Some(value) = side_store.get_mut(&size) {
                    *value = price
                };
            }
            UpdateAction::Remove => {
                side_store.remove(&size);
            }
        };
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.offers.clear();
    }

    pub fn get_price(&self, size: A) -> BidOffer<P> {
        BidOffer::new_with_price(
            self.bids
                .iter()
                .find(|(&current_size, _)| current_size >= size)
                .map(|(_, &current_price)| current_price),
            self.offers
                .iter()
                .find(|(&current_size, _)| current_size >= size)
                .map(|(_, &current_price)| current_price),
        )
    }
}

impl<P, A> Default for L2FullAmountMarketData<P, A>
where
    P: Copy + PartialOrd + Add<Output = P> + Div<Output = P> + From<i32>,
    A: Copy + Ord + PartialOrd + Add<Output = A> + Div<Output = A> + From<i32>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweepable_get_basic_price() {
        let mut test = L2SweepableMarketData::new();

        test.update(UpdateAction::Add, MarketSide::Bid, 12, 10);
        test.update(UpdateAction::Add, MarketSide::Offer, 15, 20);

        assert_eq!(
            test.get_price(1),
            BidOffer::new_with_price(Some(12), Some(15))
        );
        assert_eq!(
            test.get_price(10),
            BidOffer::new_with_price(Some(12), Some(15))
        );
        assert_eq!(test.get_price(11), BidOffer::new_with_price(None, Some(15)));
        assert_eq!(test.get_price(20), BidOffer::new_with_price(None, Some(15)));
        assert_eq!(test.get_price(21), BidOffer::new_with_price(None, None));
    }

    #[test]
    fn sweepable_get_vwap_price() {
        let mut test = L2SweepableMarketData::new();

        test.update(UpdateAction::Add, MarketSide::Bid, 12, 10);
        test.update(UpdateAction::Add, MarketSide::Bid, 10, 10);
        test.update(UpdateAction::Add, MarketSide::Bid, 8, 10);
        test.update(UpdateAction::Add, MarketSide::Bid, 6, 10);
        test.update(UpdateAction::Add, MarketSide::Offer, 16, 10);
        test.update(UpdateAction::Add, MarketSide::Offer, 20, 20);
        test.update(UpdateAction::Add, MarketSide::Offer, 24, 10);

        assert_eq!(
            test.get_price(20),
            BidOffer::new_with_price(Some(11), Some(18))
        );

        assert_eq!(
            test.get_price(40),
            BidOffer::new_with_price(Some(9), Some(20))
        );
    }

    #[test]
    fn full_amount_get_basic_price() {
        let mut test: L2FullAmountMarketData<i32, i32> = L2FullAmountMarketData::new();

        test.update(UpdateAction::Add, MarketSide::Bid, 12, 10);
        test.update(UpdateAction::Add, MarketSide::Offer, 15, 20);

        assert_eq!(
            test.get_price(1),
            BidOffer::new_with_price(Some(12), Some(15))
        );
        assert_eq!(
            test.get_price(10),
            BidOffer::new_with_price(Some(12), Some(15))
        );
        assert_eq!(test.get_price(11), BidOffer::new_with_price(None, Some(15)));
        assert_eq!(test.get_price(20), BidOffer::new_with_price(None, Some(15)));
        assert_eq!(test.get_price(21), BidOffer::new_with_price(None, None));
    }

    #[test]
    fn full_amount_get_vwap_price() {
        let mut test = L2FullAmountMarketData::new();

        test.update(UpdateAction::Add, MarketSide::Bid, 12, 10);
        test.update(UpdateAction::Add, MarketSide::Bid, 10, 20);
        test.update(UpdateAction::Add, MarketSide::Bid, 8, 30);
        test.update(UpdateAction::Add, MarketSide::Bid, 6, 40);
        test.update(UpdateAction::Add, MarketSide::Offer, 16, 10);
        test.update(UpdateAction::Add, MarketSide::Offer, 20, 30);
        test.update(UpdateAction::Add, MarketSide::Offer, 24, 40);

        assert_eq!(
            test.get_price(20),
            BidOffer::new_with_price(Some(10), Some(20))
        );

        assert_eq!(
            test.get_price(40),
            BidOffer::new_with_price(Some(6), Some(24))
        );
    }
}
